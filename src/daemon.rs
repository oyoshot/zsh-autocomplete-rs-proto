use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

use crate::app::App;
use crate::candidate::Candidate;
use crate::config::{Config, Theme};
use crate::fuzzy::FuzzyMatcher;
use crate::protocol::{self, Request, Response};
use crate::ui;
use tracing::{debug, error, info, info_span, warn};
use tracing_subscriber::EnvFilter;

struct DaemonServer {
    state: Mutex<DaemonState>,
    socket_path: PathBuf,
    shutdown_requested: AtomicBool,
}

struct DaemonState {
    theme: Theme,
    config_mtime: Option<SystemTime>,
}

pub fn start() -> io::Result<()> {
    init_tracing();

    let socket_path = protocol::socket_path();
    info!(socket_path = %socket_path.display(), "starting daemon");

    // Clean up stale socket
    if socket_path.exists() {
        if UnixStream::connect(&socket_path).is_ok() {
            warn!(socket_path = %socket_path.display(), "daemon already running");
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

    fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600))?;

    let server = Arc::new(DaemonServer {
        state: Mutex::new(DaemonState {
            theme: Config::load().theme(),
            config_mtime: config_file_mtime(),
        }),
        socket_path,
        shutdown_requested: AtomicBool::new(false),
    });

    serve(listener, Arc::clone(&server))?;

    let _ = fs::remove_file(&server.socket_path);
    info!(socket_path = %server.socket_path.display(), "daemon stopped");
    Ok(())
}

fn serve(listener: UnixListener, server: Arc<DaemonServer>) -> io::Result<()> {
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                if server.shutdown_requested.load(Ordering::SeqCst) {
                    break;
                }

                let server = Arc::clone(&server);
                thread::spawn(move || {
                    if server.handle_connection(stream) {
                        info!("shutdown requested");
                        server.shutdown_requested.store(true, Ordering::SeqCst);
                        let _ = UnixStream::connect(&server.socket_path);
                    }
                });
            }
            Err(e) => {
                if server.shutdown_requested.load(Ordering::SeqCst) {
                    break;
                }
                warn!(error = %e, "accept error");
                eprintln!("daemon: accept error: {}", e);
            }
        }
    }

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
    fn current_theme(&self) -> Theme {
        let current_mtime = config_file_mtime();
        let mut state = self
            .state
            .lock()
            .expect("daemon state mutex should not be poisoned");
        let reloaded = current_mtime != state.config_mtime;
        debug!(reloaded, "checked config reload");
        if reloaded {
            state.theme = Config::load().theme();
            state.config_mtime = current_mtime;
            info!(reloaded = true, "reloaded config");
        }
        state.theme.clone()
    }

    fn handle_connection(&self, stream: UnixStream) -> bool {
        stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
        let theme = self.current_theme();

        let mut reader = BufReader::new(&stream);

        // Peek at first byte to determine protocol
        let buf = match reader.fill_buf() {
            Ok(b) if !b.is_empty() => b[0],
            _ => {
                warn!("failed to peek request byte");
                return false;
            }
        };

        // Text protocol: first byte is ASCII letter
        // Binary protocol: first byte is part of u32 length (typically 0x00)
        if buf.is_ascii_alphabetic() {
            let _span = info_span!("request", protocol = "text").entered();
            return self.handle_text_connection(&mut reader, &stream, &theme);
        }

        // Binary protocol
        let request = match Request::deserialize(&mut reader) {
            Ok(req) => req,
            Err(e) => {
                warn!(error = %e, "failed to deserialize binary request");
                return false;
            }
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
                let _span = info_span!(
                    "render",
                    protocol = "binary",
                    prefix_len = prefix.len(),
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                    payload_bytes = candidates_tsv.len()
                )
                .entered();
                let response = self.handle_render(
                    prefix,
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                    &candidates_tsv,
                    &theme,
                );
                let mut writer = &stream;
                if let Err(e) = response.write_to(&mut writer) {
                    warn!(error = %e, "failed to write render response");
                }
                false
            }
            Request::Clear {
                popup_row,
                popup_height,
                cursor_row,
            } => {
                let _span = info_span!(
                    "clear",
                    protocol = "binary",
                    popup_row,
                    popup_height,
                    cursor_row
                )
                .entered();
                let response = self.handle_clear(popup_row, popup_height, cursor_row);
                let mut writer = &stream;
                if let Err(e) = response.write_to(&mut writer) {
                    warn!(error = %e, "failed to write clear response");
                }
                false
            }
            Request::Ping => {
                let _span = info_span!("ping", protocol = "binary").entered();
                let mut writer = &stream;
                if let Err(e) = (Response::Success {
                    tty_bytes: Vec::new(),
                    metadata: None,
                })
                .write_to(&mut writer)
                {
                    warn!(error = %e, "failed to write ping response");
                }
                false
            }
            Request::Shutdown => {
                let _span = info_span!("shutdown", protocol = "binary").entered();
                let mut writer = &stream;
                if let Err(e) = (Response::Success {
                    tty_bytes: Vec::new(),
                    metadata: None,
                })
                .write_to(&mut writer)
                {
                    warn!(error = %e, "failed to write shutdown response");
                }
                true
            }
        }
    }

    /// Text protocol for zsocket (zsh direct IPC, no subprocess spawn).
    ///
    /// Request format:
    ///   render <row> <col> <cols> <rows>\n
    ///   <prefix>\n
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
        &self,
        reader: &mut BufReader<&UnixStream>,
        stream: &UnixStream,
        theme: &Theme,
    ) -> bool {
        let mut header = String::new();
        if reader.read_line(&mut header).is_err() {
            warn!("failed to read text request header");
            return false;
        }
        let header = header.trim_end();
        let parts: Vec<&str> = header.split(' ').collect();

        if parts.is_empty() {
            warn!("received empty text request");
            return false;
        }

        let mut writer = io::BufWriter::new(stream);

        match parts[0] {
            "render" if parts.len() == 5 => {
                let cursor_row: u16 = parts[1].parse().unwrap_or(0);
                let cursor_col: u16 = parts[2].parse().unwrap_or(0);
                let term_cols: u16 = parts[3].parse().unwrap_or(80);
                let term_rows: u16 = parts[4].parse().unwrap_or(24);
                let prefix = match read_text_line(reader) {
                    Ok(prefix) => prefix,
                    Err(_) => {
                        warn!("invalid text render prefix");
                        let _ = writeln!(writer, "ERROR invalid prefix");
                        let _ = writer.flush();
                        return false;
                    }
                };

                // Read candidates until END line (max 1MB)
                const MAX_TSV_BYTES: usize = 1_048_576;
                let mut tsv = String::new();
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).is_err() || line.is_empty() {
                        break;
                    }
                    if line.trim_end() == "END" {
                        break;
                    }
                    if tsv.len() + line.len() > MAX_TSV_BYTES {
                        // Drain remaining lines until END
                        loop {
                            let mut drain = String::new();
                            if reader.read_line(&mut drain).is_err()
                                || drain.is_empty()
                                || drain.trim_end() == "END"
                            {
                                break;
                            }
                        }
                        let _ = writeln!(writer, "ERROR payload too large");
                        let _ = writer.flush();
                        return false;
                    }
                    tsv.push_str(&line);
                }

                let _span = info_span!(
                    "render",
                    protocol = "text",
                    prefix_len = prefix.len(),
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                    payload_bytes = tsv.len()
                )
                .entered();
                let response = self.handle_render(
                    prefix,
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                    tsv.as_bytes(),
                    theme,
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

                let _span = info_span!(
                    "clear",
                    protocol = "text",
                    popup_row,
                    popup_height,
                    cursor_row
                )
                .entered();
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
                let _span = info_span!("ping", protocol = "text").entered();
                let _ = writeln!(writer, "OK");
                let _ = writer.flush();
                false
            }
            "shutdown" => {
                let _span = info_span!("shutdown", protocol = "text").entered();
                let _ = writeln!(writer, "OK");
                let _ = writer.flush();
                true
            }
            _ => {
                warn!(header = header, "unknown text request");
                false
            }
        }
    }

    fn handle_render(
        &self,
        prefix: String,
        cursor_row: u16,
        cursor_col: u16,
        term_cols: u16,
        term_rows: u16,
        candidates_tsv: &[u8],
        theme: &Theme,
    ) -> Response {
        let tsv_str = match std::str::from_utf8(candidates_tsv) {
            Ok(s) => s,
            Err(e) => {
                error!(error = %e, "invalid UTF-8 render payload");
                return Response::Error(format!("invalid UTF-8: {}", e));
            }
        };

        let candidates: Vec<Candidate> = tsv_str
            .lines()
            .filter(|line| !line.is_empty())
            .map(Candidate::parse_line)
            .collect();

        if candidates.is_empty() {
            debug!("render request had no candidates");
            return Response::Empty;
        }

        let mut app = App::new_with_matcher(
            candidates,
            prefix,
            cursor_row,
            cursor_col,
            term_cols,
            term_rows,
            FuzzyMatcher::new(),
        );

        // Cap max_visible based on terminal rows
        let max_popup_height = term_rows.saturating_sub(1);
        if app.max_visible as u16 + 2 > max_popup_height {
            app.max_visible = max_popup_height.saturating_sub(2).max(1) as usize;
        }

        if app.max_visible == 0 {
            debug!("render request had zero visible rows");
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
        let result = ui::render::render_popup_to_bytes(&app, theme);

        match result {
            Ok((mut tty_bytes, popup)) => {
                if !scroll_bytes.is_empty() {
                    let mut combined = scroll_bytes;
                    combined.append(&mut tty_bytes);
                    tty_bytes = combined;
                }
                debug!(
                    popup_row = popup.row,
                    popup_height = popup.height,
                    cursor_row = cursor_row_final,
                    tty_bytes = tty_bytes.len(),
                    "render complete"
                );
                let metadata = format!(
                    "popup_row={} popup_height={} cursor_row={}",
                    popup.row, popup.height, cursor_row_final
                );
                Response::Success {
                    tty_bytes,
                    metadata: Some(metadata),
                }
            }
            Err(e) => {
                error!(error = %e, "render failed");
                Response::Error(format!("render failed: {}", e))
            }
        }
    }

    fn handle_clear(&self, popup_row: u16, popup_height: u16, cursor_row: u16) -> Response {
        match ui::render::clear_rect_to_bytes(popup_row, popup_height, cursor_row) {
            Ok(tty_bytes) => {
                debug!(tty_bytes = tty_bytes.len(), "clear complete");
                Response::Success {
                    tty_bytes,
                    metadata: None,
                }
            }
            Err(e) => {
                error!(error = %e, "clear failed");
                Response::Error(format!("clear failed: {}", e))
            }
        }
    }
}

fn init_tracing() {
    if std::env::var_os("RUST_LOG").is_none() {
        return;
    }

    let filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(e) => {
            eprintln!("daemon: invalid RUST_LOG: {}", e);
            return;
        }
    };

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();
}

fn config_file_mtime() -> Option<SystemTime> {
    let path = dirs::config_dir()?.join("zacrs").join("config.toml");
    fs::metadata(path).ok()?.modified().ok()
}

fn read_text_line(reader: &mut impl BufRead) -> io::Result<String> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    Ok(line)
}

#[cfg(test)]
mod tests {
    use super::{DaemonServer, DaemonState, read_text_line, serve};
    use crate::protocol::{Request, Response};
    use std::fs;
    use std::io::{self, BufReader, Cursor, Write};
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::PathBuf;
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn read_text_line_preserves_spaces() {
        let data = Cursor::new(b"  My Dir  \n");
        let mut reader = BufReader::new(data);

        let line = read_text_line(&mut reader).unwrap();

        assert_eq!(line, "  My Dir  ");
    }

    #[test]
    fn read_text_line_handles_empty_line() {
        let data = Cursor::new(b"\n");
        let mut reader = BufReader::new(data);

        let line = read_text_line(&mut reader).unwrap();

        assert_eq!(line, "");
    }

    fn unique_socket_path(name: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PathBuf::from(format!(
            "target/test-sockets/z-{}-{}-{}.sock",
            name,
            std::process::id(),
            timestamp
        ))
    }

    fn start_test_server(name: &str) -> (PathBuf, thread::JoinHandle<io::Result<()>>) {
        let socket_path = unique_socket_path(name);
        fs::create_dir_all(socket_path.parent().unwrap()).unwrap();
        let listener = UnixListener::bind(&socket_path).unwrap();

        let server = Arc::new(DaemonServer {
            state: Mutex::new(DaemonState {
                theme: crate::config::Theme::default(),
                config_mtime: None,
            }),
            socket_path: socket_path.clone(),
            shutdown_requested: AtomicBool::new(false),
        });

        let handle = thread::spawn({
            let server = Arc::clone(&server);
            move || {
                let result = serve(listener, Arc::clone(&server));
                let _ = fs::remove_file(&server.socket_path);
                result
            }
        });

        (socket_path, handle)
    }

    fn send_shutdown(socket_path: &PathBuf) {
        let stream = UnixStream::connect(socket_path).unwrap();
        let mut writer = &stream;
        writer.write_all(&Request::Shutdown.serialize()).unwrap();
        let mut reader = BufReader::new(&stream);
        let response = Response::deserialize(&mut reader).unwrap();
        assert!(matches!(response, Response::Success { .. }));
    }

    #[test]
    fn slow_client_does_not_block_other_requests() {
        let (socket_path, handle) = start_test_server("slow-client");

        let mut slow_client = UnixStream::connect(&socket_path).unwrap();
        slow_client
            .write_all(b"render 0 0 80 24\nprefix\ncandidate\t\t\n")
            .unwrap();

        thread::sleep(Duration::from_millis(50));

        let ping_client = UnixStream::connect(&socket_path).unwrap();
        ping_client
            .set_read_timeout(Some(Duration::from_millis(500)))
            .unwrap();
        let mut writer = &ping_client;
        writer.write_all(&Request::Ping.serialize()).unwrap();
        let mut reader = BufReader::new(&ping_client);
        let response = Response::deserialize(&mut reader).unwrap();
        assert!(matches!(response, Response::Success { .. }));

        drop(slow_client);
        send_shutdown(&socket_path);
        handle.join().unwrap().unwrap();
    }

    #[test]
    fn shutdown_request_stops_accept_loop() {
        let (socket_path, handle) = start_test_server("shutdown");

        send_shutdown(&socket_path);

        handle.join().unwrap().unwrap();
    }
}
