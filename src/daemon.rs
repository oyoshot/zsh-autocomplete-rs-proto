use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::time::SystemTime;

use crate::app::App;
use crate::candidate::Candidate;
use crate::config::{Config, Theme};
use crate::fuzzy::FuzzyMatcher;
use crate::protocol::{self, Request, Response};
use crate::ui;

struct DaemonServer {
    config: Config,
    theme: Theme,
    config_mtime: Option<SystemTime>,
    socket_path: PathBuf,
    fuzzy: Option<FuzzyMatcher>,
}

pub fn start() -> io::Result<()> {
    let socket_path = protocol::socket_path();

    // Clean up stale socket
    if socket_path.exists() {
        if UnixStream::connect(&socket_path).is_ok() {
            return Err(io::Error::new(
                io::ErrorKind::AddrInUse,
                "daemon already running",
            ));
        }
        fs::remove_file(&socket_path)?;
    }

    if let Some(parent) = socket_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let listener = UnixListener::bind(&socket_path)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600))?;
    }

    let config = Config::load();
    let config_mtime = config_file_mtime();
    let theme = config.theme();

    let mut server = DaemonServer {
        config,
        theme,
        config_mtime,
        socket_path,
        fuzzy: Some(FuzzyMatcher::new()),
    };

    eprintln!("daemon: listening on {}", server.socket_path.display());

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                server.maybe_reload_config();
                if server.handle_connection(stream) {
                    break; // shutdown requested
                }
            }
            Err(e) => {
                eprintln!("daemon: accept error: {}", e);
            }
        }
    }

    let _ = fs::remove_file(&server.socket_path);
    eprintln!("daemon: stopped");
    Ok(())
}

pub fn stop() -> io::Result<()> {
    let socket_path = protocol::socket_path();
    let stream = UnixStream::connect(&socket_path)
        .map_err(|_| io::Error::new(io::ErrorKind::NotConnected, "daemon not running"))?;
    let mut writer = &stream;
    writer.write_all(&Request::Shutdown.serialize())?;
    let mut reader = BufReader::new(&stream);
    let _ = Response::deserialize(&mut reader);
    Ok(())
}

pub fn status() -> bool {
    let socket_path = protocol::socket_path();
    let Ok(stream) = UnixStream::connect(&socket_path) else {
        return false;
    };
    let mut writer = &stream;
    if writer.write_all(&Request::Ping.serialize()).is_err() {
        return false;
    }
    let mut reader = BufReader::new(&stream);
    Response::deserialize(&mut reader).is_ok()
}

impl DaemonServer {
    fn maybe_reload_config(&mut self) {
        let current_mtime = config_file_mtime();
        if current_mtime != self.config_mtime {
            self.config = Config::load();
            self.theme = self.config.theme();
            self.config_mtime = current_mtime;
        }
    }

    fn handle_connection(&mut self, stream: UnixStream) -> bool {
        use std::time::Duration;
        stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(5))).ok();

        let mut reader = BufReader::new(&stream);

        // Peek at first byte to determine protocol
        let buf = match reader.fill_buf() {
            Ok(b) if !b.is_empty() => b[0],
            _ => return false,
        };

        // Text protocol: first byte is ASCII letter
        // Binary protocol: first byte is part of u32 length (typically 0x00)
        if buf.is_ascii_alphabetic() {
            return self.handle_text_connection(&mut reader, &stream);
        }

        // Binary protocol
        let request = match Request::deserialize(&mut reader) {
            Ok(req) => req,
            Err(_) => return false,
        };

        match request {
            Request::Render {
                prefix,
                cursor_row,
                cursor_col,
                term_cols,
                term_rows,
                candidates_tsv,
            } => {
                let response = self.handle_render(
                    prefix,
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                    &candidates_tsv,
                );
                let mut writer = &stream;
                let _ = response.write_to(&mut writer);
                false
            }
            Request::Clear {
                popup_row,
                popup_height,
                cursor_row,
            } => {
                let response = self.handle_clear(popup_row, popup_height, cursor_row);
                let mut writer = &stream;
                let _ = response.write_to(&mut writer);
                false
            }
            Request::Ping => {
                let mut writer = &stream;
                let _ = Response::Success {
                    tty_bytes: Vec::new(),
                    metadata: None,
                }
                .write_to(&mut writer);
                false
            }
            Request::Shutdown => {
                let mut writer = &stream;
                let _ = Response::Success {
                    tty_bytes: Vec::new(),
                    metadata: None,
                }
                .write_to(&mut writer);
                true
            }
        }
    }

    /// Text protocol for zsocket (zsh direct IPC, no subprocess spawn).
    ///
    /// Request format:
    ///   render <prefix> <row> <col> <cols> <rows>\n
    ///   <candidates_tsv lines...>\n
    ///   END\n
    ///
    ///   clear <popup_row> <popup_height> <cursor_row>\n
    ///   ping\n
    ///   shutdown\n
    ///
    /// Response format (render success):
    ///   OK <popup_row> <popup_height> <cursor_row> <tty_len>\n
    ///   <tty_bytes, exactly tty_len bytes>
    ///
    /// Response format (empty/error):
    ///   EMPTY\n
    ///   ERROR <message>\n
    fn handle_text_connection(
        &mut self,
        reader: &mut BufReader<&UnixStream>,
        stream: &UnixStream,
    ) -> bool {
        let mut header = String::new();
        if reader.read_line(&mut header).is_err() {
            return false;
        }
        let header = header.trim_end();
        let parts: Vec<&str> = header.split(' ').collect();

        if parts.is_empty() {
            return false;
        }

        let mut writer = io::BufWriter::new(stream);

        match parts[0] {
            "render" if parts.len() == 6 => {
                let prefix = parts[1].to_string();
                let cursor_row: u16 = parts[2].parse().unwrap_or(0);
                let cursor_col: u16 = parts[3].parse().unwrap_or(0);
                let term_cols: u16 = parts[4].parse().unwrap_or(80);
                let term_rows: u16 = parts[5].parse().unwrap_or(24);

                // Read candidates until END line
                let mut tsv = String::new();
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).is_err() || line.is_empty() {
                        break;
                    }
                    if line.trim_end() == "END" {
                        break;
                    }
                    tsv.push_str(&line);
                }

                let response = self.handle_render(
                    prefix,
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                    tsv.as_bytes(),
                );

                match response {
                    Response::Success {
                        tty_bytes,
                        metadata,
                    } => {
                        let meta = metadata.unwrap_or_default();
                        let _ = writeln!(writer, "OK {} {}", meta, tty_bytes.len());
                        let _ = writer.write_all(&tty_bytes);
                        let _ = writer.flush();
                    }
                    Response::Empty => {
                        let _ = writeln!(writer, "EMPTY");
                    }
                    Response::Error(msg) => {
                        let _ = writeln!(writer, "ERROR {}", msg);
                    }
                }
                false
            }
            "clear" if parts.len() == 4 => {
                let popup_row: u16 = parts[1].parse().unwrap_or(0);
                let popup_height: u16 = parts[2].parse().unwrap_or(0);
                let cursor_row: u16 = parts[3].parse().unwrap_or(0);

                let response = self.handle_clear(popup_row, popup_height, cursor_row);
                match response {
                    Response::Success { tty_bytes, .. } => {
                        let _ = writeln!(writer, "OK {}", tty_bytes.len());
                        let _ = writer.write_all(&tty_bytes);
                        let _ = writer.flush();
                    }
                    _ => {
                        let _ = writeln!(writer, "ERROR clear failed");
                    }
                }
                false
            }
            "ping" => {
                let _ = writeln!(writer, "OK");
                let _ = writer.flush();
                false
            }
            "shutdown" => {
                let _ = writeln!(writer, "OK");
                let _ = writer.flush();
                true
            }
            _ => false,
        }
    }

    fn handle_render(
        &mut self,
        prefix: String,
        cursor_row: u16,
        cursor_col: u16,
        term_cols: u16,
        term_rows: u16,
        candidates_tsv: &[u8],
    ) -> Response {
        let tsv_str = match std::str::from_utf8(candidates_tsv) {
            Ok(s) => s,
            Err(e) => return Response::Error(format!("invalid UTF-8: {}", e)),
        };

        let candidates: Vec<Candidate> = tsv_str
            .lines()
            .filter(|line| !line.is_empty())
            .map(Candidate::parse_line)
            .collect();

        if candidates.is_empty() {
            return Response::Empty;
        }

        // Take fuzzy matcher out to reuse, put it back after
        let fuzzy = self.fuzzy.take().unwrap_or_default();
        let mut app = App::new_with_matcher(
            candidates, prefix, cursor_row, cursor_col, term_cols, term_rows, fuzzy,
        );

        // Cap max_visible based on terminal rows
        let max_popup_height = term_rows.saturating_sub(1);
        if app.max_visible as u16 + 2 > max_popup_height {
            app.max_visible = max_popup_height.saturating_sub(2).max(1) as usize;
        }

        if app.max_visible == 0 {
            return Response::Empty;
        }

        // ensure_space: compute scroll amount and adjust cursor_row
        let popup_height = app.max_visible as u16 + 2;
        let space_below = term_rows.saturating_sub(app.cursor_row + 1);
        let mut scroll_bytes = Vec::new();
        if space_below < popup_height {
            let scroll_amount = (popup_height - space_below).min(app.cursor_row);
            if scroll_amount > 0 {
                let _ = crossterm::queue!(
                    &mut scroll_bytes,
                    crossterm::terminal::ScrollUp(scroll_amount)
                );
                app.cursor_row -= scroll_amount;
            }
        }

        // Read cursor_row before render to avoid borrowing app after render
        let cursor_row_final = app.cursor_row;
        let result = ui::render::render_popup_to_bytes(&app, &self.theme);

        // Reclaim the FuzzyMatcher for reuse
        self.fuzzy = Some(app.take_fuzzy());

        match result {
            Ok((mut tty_bytes, popup)) => {
                if !scroll_bytes.is_empty() {
                    let mut combined = scroll_bytes;
                    combined.append(&mut tty_bytes);
                    tty_bytes = combined;
                }
                let metadata = format!(
                    "popup_row={} popup_height={} cursor_row={}",
                    popup.row, popup.height, cursor_row_final
                );
                Response::Success {
                    tty_bytes,
                    metadata: Some(metadata),
                }
            }
            Err(e) => Response::Error(format!("render failed: {}", e)),
        }
    }

    fn handle_clear(&self, popup_row: u16, popup_height: u16, cursor_row: u16) -> Response {
        match ui::render::clear_rect_to_bytes(popup_row, popup_height, cursor_row) {
            Ok(tty_bytes) => Response::Success {
                tty_bytes,
                metadata: None,
            },
            Err(e) => Response::Error(format!("clear failed: {}", e)),
        }
    }
}

fn config_file_mtime() -> Option<SystemTime> {
    let path = dirs::config_dir()?.join("zacrs").join("config.toml");
    fs::metadata(path).ok()?.modified().ok()
}
