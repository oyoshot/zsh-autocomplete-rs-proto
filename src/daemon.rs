use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::time::SystemTime;

use crate::app::App;
use crate::candidate::Candidate;
use crate::config::{Config, KeyBindings, Theme};
use crate::fuzzy::FuzzyMatcher;
use crate::input::{self, Action};
use crate::protocol::{self, Request, Response};
use crate::ui;
use tracing::{debug, error, info, info_span, warn};
use tracing_subscriber::EnvFilter;

struct DaemonServer {
    config: Config,
    theme: Theme,
    key_bindings: KeyBindings,
    config_mtime: Option<SystemTime>,
    socket_path: PathBuf,
    fuzzy: Option<FuzzyMatcher>,
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

    let config = Config::load();
    let config_mtime = config_file_mtime();
    let theme = config.theme();
    let key_bindings = config.key_bindings();

    let mut server = DaemonServer {
        config,
        theme,
        key_bindings,
        config_mtime,
        socket_path,
        fuzzy: Some(FuzzyMatcher::new()),
    };

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                server.maybe_reload_config();
                if server.handle_connection(stream) {
                    info!("shutdown requested");
                    break; // shutdown requested
                }
            }
            Err(e) => {
                warn!(error = %e, "accept error");
                eprintln!("daemon: accept error: {}", e);
            }
        }
    }

    let _ = fs::remove_file(&server.socket_path);
    info!(socket_path = %server.socket_path.display(), "daemon stopped");
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
        let reloaded = current_mtime != self.config_mtime;
        debug!(reloaded, "checked config reload");
        if reloaded {
            self.config = Config::load();
            self.theme = self.config.theme();
            self.key_bindings = self.config.key_bindings();
            self.config_mtime = current_mtime;
            info!(reloaded = true, "reloaded config");
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
            _ => {
                warn!("failed to peek request byte");
                return false;
            }
        };

        // Text protocol: first byte is ASCII letter
        // Binary protocol: first byte is part of u32 length (typically 0x00)
        if buf.is_ascii_alphabetic() {
            let _span = info_span!("request", protocol = "text").entered();
            return self.handle_text_connection(&mut reader, &stream);
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
    ///   complete <row> <col> <cols> <rows> [reuse=1]\n
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
        &mut self,
        reader: &mut BufReader<&UnixStream>,
        stream: &UnixStream,
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

                let tsv = match read_tsv_payload(reader) {
                    Ok(tsv) => tsv,
                    Err(msg) => {
                        let _ = writeln!(writer, "ERROR {}", msg);
                        let _ = writer.flush();
                        return false;
                    }
                };

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
            "complete" => {
                let Some(reuse_initial_frame) = parse_complete_reuse(&parts) else {
                    warn!(header = header, "invalid text complete request");
                    let _ = writeln!(writer, "ERROR invalid complete request");
                    let _ = writer.flush();
                    return false;
                };
                let cursor_row: u16 = parts[1].parse().unwrap_or(0);
                let cursor_col: u16 = parts[2].parse().unwrap_or(0);
                let term_cols: u16 = parts[3].parse().unwrap_or(80);
                let term_rows: u16 = parts[4].parse().unwrap_or(24);
                let prefix = match read_text_line(reader) {
                    Ok(prefix) => prefix,
                    Err(_) => {
                        warn!("invalid text complete prefix");
                        let _ = writeln!(writer, "ERROR invalid prefix");
                        let _ = writer.flush();
                        return false;
                    }
                };

                let tsv = match read_tsv_payload(reader) {
                    Ok(tsv) => tsv,
                    Err(msg) => {
                        let _ = writeln!(writer, "ERROR {}", msg);
                        let _ = writer.flush();
                        return false;
                    }
                };

                let _span = info_span!(
                    "complete",
                    protocol = "text",
                    prefix_len = prefix.len(),
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                    reuse_initial_frame,
                    payload_bytes = tsv.len()
                )
                .entered();

                // Extend timeout for interactive session
                use std::time::Duration;
                stream.set_read_timeout(Some(Duration::from_secs(60))).ok();

                self.handle_complete(
                    reader,
                    &mut writer,
                    prefix,
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                    &tsv,
                    reuse_initial_frame,
                );
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

    #[allow(clippy::too_many_arguments)]
    fn handle_complete(
        &mut self,
        reader: &mut BufReader<&UnixStream>,
        writer: &mut io::BufWriter<&UnixStream>,
        prefix: String,
        cursor_row: u16,
        cursor_col: u16,
        term_cols: u16,
        term_rows: u16,
        tsv: &str,
        reuse_initial_frame: bool,
    ) {
        let candidates: Vec<Candidate> = tsv
            .lines()
            .filter(|line| !line.is_empty())
            .map(Candidate::parse_line)
            .collect();

        if candidates.is_empty() {
            let _ = writeln!(writer, "DONE 1 ");
            let _ = writer.flush();
            return;
        }

        let fuzzy = self.fuzzy.take().unwrap_or_default();
        let mut app = App::new_with_matcher(
            candidates, prefix, cursor_row, cursor_col, term_cols, term_rows, fuzzy,
        );

        // Cap max_visible
        let max_popup_height = term_rows.saturating_sub(1);
        if app.max_visible as u16 + 2 > max_popup_height {
            app.max_visible = max_popup_height.saturating_sub(2).max(1) as usize;
        }

        if app.max_visible == 0 {
            self.fuzzy = Some(app.take_fuzzy());
            let _ = writeln!(writer, "DONE 1 ");
            let _ = writer.flush();
            return;
        }

        // ensure_space: compute scroll bytes
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

        // Initial frame
        let send_frame = |writer: &mut io::BufWriter<&UnixStream>,
                          app: &App,
                          theme: &Theme,
                          extra_prefix: &[u8]|
         -> io::Result<()> {
            let (tty_bytes, popup) = ui::render::draw_to_bytes(app, theme)?;
            let total_len = extra_prefix.len() + tty_bytes.len();
            writeln!(
                writer,
                "FRAME popup_row={} popup_height={} cursor_row={} {}",
                popup.row, popup.height, app.cursor_row, total_len
            )?;
            if !extra_prefix.is_empty() {
                writer.write_all(extra_prefix)?;
            }
            writer.write_all(&tty_bytes)?;
            writer.flush()
        };

        let can_reuse_initial_frame =
            reuse_initial_frame && scroll_bytes.is_empty() && app.filter_text == app.prefix;
        let initial_frame_result = if can_reuse_initial_frame {
            writeln!(writer, "NONE").and_then(|_| writer.flush())
        } else {
            send_frame(writer, &app, &self.theme, &scroll_bytes)
        };

        if initial_frame_result.is_err() {
            self.fuzzy = Some(app.take_fuzzy());
            return;
        }

        // Interactive loop
        let bindings = &self.key_bindings;
        let theme = &self.theme;

        loop {
            let mut msg_line = String::new();
            if reader.read_line(&mut msg_line).is_err() || msg_line.is_empty() {
                break; // Connection closed
            }
            let msg_line = msg_line.trim_end();

            if let Some(len_str) = msg_line.strip_prefix("KEY ") {
                let byte_count: usize = len_str.parse().unwrap_or(0);
                if byte_count == 0 || byte_count > 16 {
                    let _ = writeln!(writer, "NONE");
                    let _ = writer.flush();
                    continue;
                }
                let mut key_buf = vec![0u8; byte_count];
                if io::Read::read_exact(reader, &mut key_buf).is_err() {
                    break;
                }

                let action = input::parse_raw_bytes(&key_buf, bindings);

                match action {
                    Action::MoveDown | Action::MoveUp | Action::PageDown | Action::PageUp => {
                        match action {
                            Action::MoveDown => app.move_down(),
                            Action::MoveUp => app.move_up(),
                            Action::PageDown => app.page_down(),
                            Action::PageUp => app.page_up(),
                            _ => unreachable!(),
                        }
                        if send_frame(writer, &app, theme, &[]).is_err() {
                            break;
                        }
                    }
                    Action::TypeChar(c) => {
                        let clear_bytes = ui::render::clear_to_bytes(&app).unwrap_or_default();
                        app.type_char(c);
                        if app.filtered_indices.is_empty() {
                            let _ = writeln!(writer, "DONE 1 {}", app.filter_text);
                            let _ = writer.flush();
                            break;
                        }
                        if send_frame(writer, &app, theme, &clear_bytes).is_err() {
                            break;
                        }
                    }
                    Action::Backspace => {
                        let clear_bytes = ui::render::clear_to_bytes(&app).unwrap_or_default();
                        if !app.backspace() {
                            let _ = writeln!(writer, "DONE 1 ");
                            let _ = writer.flush();
                            break;
                        }
                        if app.filtered_indices.is_empty()
                            || app.filter_text.len() < app.prefix.len()
                        {
                            let _ = writeln!(writer, "DONE 1 {}", app.filter_text);
                            let _ = writer.flush();
                            break;
                        }
                        if send_frame(writer, &app, theme, &clear_bytes).is_err() {
                            break;
                        }
                    }
                    Action::Confirm => {
                        let result = app
                            .selected_candidate()
                            .map(|c| c.text_with_suffix())
                            .unwrap_or_default();
                        let _ = writeln!(writer, "DONE 0 {}", result);
                        let _ = writer.flush();
                        break;
                    }
                    Action::DismissWithSpace => {
                        let _ = writeln!(writer, "DONE 2 {} ", app.filter_text);
                        let _ = writer.flush();
                        break;
                    }
                    Action::Cancel => {
                        let text = if app.filter_text != app.prefix {
                            &app.filter_text
                        } else {
                            ""
                        };
                        let _ = writeln!(writer, "DONE 1 {}", text);
                        let _ = writer.flush();
                        break;
                    }
                    Action::Resize(_, _) | Action::None => {
                        let _ = writeln!(writer, "NONE");
                        let _ = writer.flush();
                    }
                }
            } else {
                // Unknown message, ignore
                let _ = writeln!(writer, "NONE");
                let _ = writer.flush();
            }
        }

        self.fuzzy = Some(app.take_fuzzy());
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

fn read_tsv_payload(reader: &mut impl BufRead) -> Result<String, String> {
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
            return Err("payload too large".to_string());
        }
        tsv.push_str(&line);
    }
    Ok(tsv)
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

fn parse_complete_reuse(parts: &[&str]) -> Option<bool> {
    match parts {
        ["complete", _, _, _, _] => Some(false),
        ["complete", _, _, _, _, "reuse=1"] => Some(true),
        ["complete", _, _, _, _, "reuse=0"] => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{DaemonServer, parse_complete_reuse, read_text_line};
    use crate::config::Config;
    use crate::fuzzy::FuzzyMatcher;
    use std::io::{BufRead, BufReader, Cursor};
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;
    use std::thread;

    fn test_server() -> DaemonServer {
        let config = Config::default();
        let theme = config.theme();
        let key_bindings = config.key_bindings();
        DaemonServer {
            config,
            theme,
            key_bindings,
            config_mtime: None,
            socket_path: PathBuf::from("/tmp/zacrs-test.sock"),
            fuzzy: Some(FuzzyMatcher::new()),
        }
    }

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

    #[test]
    fn parse_complete_reuse_accepts_optional_flag() {
        assert_eq!(
            parse_complete_reuse(&["complete", "1", "2", "80", "24"]),
            Some(false)
        );
        assert_eq!(
            parse_complete_reuse(&["complete", "1", "2", "80", "24", "reuse=1"]),
            Some(true)
        );
        assert_eq!(
            parse_complete_reuse(&["complete", "1", "2", "80", "24", "reuse=0"]),
            Some(false)
        );
        assert_eq!(
            parse_complete_reuse(&["complete", "1", "2", "80", "24", "unexpected"]),
            None
        );
    }

    #[test]
    fn handle_complete_reuse_sends_none_before_interaction() {
        let (server_stream, client_stream) = UnixStream::pair().unwrap();
        let handle = thread::spawn(move || {
            let mut server = test_server();
            let mut reader = BufReader::new(&server_stream);
            let mut writer = std::io::BufWriter::new(&server_stream);
            server.handle_complete(
                &mut reader,
                &mut writer,
                "gi".to_string(),
                5,
                2,
                80,
                24,
                "git\tcommand\tcommand\ngizmo\tcommand\tcommand\n",
                true,
            );
        });

        let mut reader = BufReader::new(&client_stream);
        let mut header = String::new();
        reader.read_line(&mut header).unwrap();
        assert_eq!(header.trim_end(), "NONE");

        drop(reader);
        drop(client_stream);
        handle.join().unwrap();
    }

    #[test]
    fn handle_complete_reuse_sends_frame_when_common_prefix_expands() {
        let (server_stream, client_stream) = UnixStream::pair().unwrap();
        let handle = thread::spawn(move || {
            let mut server = test_server();
            let mut reader = BufReader::new(&server_stream);
            let mut writer = std::io::BufWriter::new(&server_stream);
            server.handle_complete(
                &mut reader,
                &mut writer,
                "gi".to_string(),
                5,
                2,
                80,
                24,
                "git\tcommand\tcommand\n",
                true,
            );
        });

        let mut reader = BufReader::new(&client_stream);
        let mut header = String::new();
        reader.read_line(&mut header).unwrap();
        assert!(header.starts_with("FRAME "));

        drop(reader);
        drop(client_stream);
        handle.join().unwrap();
    }
}
