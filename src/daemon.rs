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
use crate::handoff::compute_reuse_token;
use crate::input::{self, Action};
use crate::protocol::{self, Request, Response};
use crate::ui;
use tracing::{debug, error, info, info_span, warn};
use tracing_subscriber::EnvFilter;

const MAX_INLINE_KEY_BYTES: usize = 16;

struct RenderParams {
    prefix: String,
    cursor_row: u16,
    cursor_col: u16,
    term_cols: u16,
    term_rows: u16,
    selected: Option<usize>,
}

struct CompleteParams {
    prefix: String,
    cursor_row: u16,
    cursor_col: u16,
    term_cols: u16,
    term_rows: u16,
    reuse_popup: bool,
    shift_tab_sequence: Option<Vec<u8>>,
}

struct DaemonServer {
    config: Config,
    theme: Theme,
    key_bindings: KeyBindings,
    config_mtime: Option<SystemTime>,
    socket_path: PathBuf,
    fuzzy: Option<FuzzyMatcher>,
}

impl DaemonServer {
    fn new(socket_path: PathBuf) -> Self {
        let config = Config::load();
        let config_mtime = config_file_mtime();
        let theme = config.theme();
        let key_bindings = config.key_bindings();
        Self {
            config,
            theme,
            key_bindings,
            config_mtime,
            socket_path,
            fuzzy: Some(FuzzyMatcher::new()),
        }
    }
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

    let mut server = DaemonServer::new(socket_path);

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
                selected,
            } => {
                let _span = info_span!(
                    "render",
                    protocol = "binary",
                    prefix_len = prefix.len(),
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                    ?selected,
                    payload_bytes = candidates_tsv.len()
                )
                .entered();
                let response = self.handle_render(
                    RenderParams {
                        prefix,
                        cursor_row,
                        cursor_col,
                        term_cols,
                        term_rows,
                        selected: selected.map(usize::from),
                    },
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
    ///   complete <row> <col> <cols> <rows> [reuse_token=<id>] [shift_tab_hex=<hex>]\n
    ///   <prefix>\n
    ///   <candidates_tsv lines...>\n
    ///   END\n
    ///
    ///   clear <popup_row> <popup_height> <cursor_row>\n
    ///   ping\n
    ///   shutdown\n
    ///
    /// Response format (render success):
    ///   OK <popup_row> <popup_height> <cursor_row> reuse_token=<id> <tty_len>\n
    ///   <tty_bytes, exactly tty_len bytes>
    ///
    /// Response format (empty/error):
    ///   EMPTY\n
    ///   ERROR <message>\n
    ///
    /// Popup-session responses (on the persistent connection):
    ///   FRAME popup_row=<N> popup_height=<N> cursor_row=<N> <tty_len>\n<tty_bytes>
    ///   DONE <exit_code> <text>\n
    ///     exit_code: 0=Confirm, 1=Cancel, 2=DismissWithSpace, 3=Passthrough
    ///   NONE\n
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
            "render" if parts.len() >= 5 => {
                let (cursor_row, cursor_col, term_cols, term_rows) = parse_terminal_dims(&parts);
                let selected: Option<usize> = parts[5..]
                    .iter()
                    .find_map(|part| part.strip_prefix("selected="))
                    .and_then(|v| v.parse().ok());
                let (prefix, tsv) = match read_prefix_and_tsv(reader, &mut writer, "render") {
                    Ok(v) => v,
                    Err(()) => return false,
                };

                let _span = info_span!(
                    "render",
                    protocol = "text",
                    prefix_len = prefix.len(),
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                    ?selected,
                    payload_bytes = tsv.len()
                )
                .entered();
                let response = self.handle_render(
                    RenderParams {
                        prefix,
                        cursor_row,
                        cursor_col,
                        term_cols,
                        term_rows,
                        selected,
                    },
                    tsv.as_bytes(),
                );

                match response {
                    Response::Success {
                        tty_bytes,
                        metadata,
                    } => {
                        let meta = metadata.unwrap_or_default();
                        let _ = writeln!(writer, "OK {} tty_len={}", meta, tty_bytes.len());
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
            "complete" if parts.len() >= 5 => {
                let (cursor_row, cursor_col, term_cols, term_rows) = parse_terminal_dims(&parts);
                let reuse_popup = parts[5..]
                    .iter()
                    .any(|part| part.starts_with("reuse_token="));
                let shift_tab_sequence = parts[5..]
                    .iter()
                    .find_map(|part| part.strip_prefix("shift_tab_hex="))
                    .and_then(crate::protocol::decode_hex_bytes);
                let (prefix, tsv) = match read_prefix_and_tsv(reader, &mut writer, "complete") {
                    Ok(v) => v,
                    Err(()) => return false,
                };

                let _span = info_span!(
                    "complete",
                    protocol = "text",
                    prefix_len = prefix.len(),
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                    reuse_popup,
                    extra_parts = parts.len().saturating_sub(5),
                    payload_bytes = tsv.len()
                )
                .entered();

                // Extend timeout for interactive session
                use std::time::Duration;
                stream.set_read_timeout(Some(Duration::from_secs(60))).ok();

                self.handle_complete(
                    reader,
                    &mut writer,
                    CompleteParams {
                        prefix,
                        cursor_row,
                        cursor_col,
                        term_cols,
                        term_rows,
                        reuse_popup,
                        shift_tab_sequence,
                    },
                    &tsv,
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

    fn send_frame<W: Write>(
        &self,
        writer: &mut W,
        app: &App,
        extra_prefix: &[u8],
        popup_only: bool,
    ) -> io::Result<()> {
        let (tty_bytes, popup) = if popup_only {
            ui::render::render_popup_to_bytes(app, &self.theme)?
        } else {
            ui::render::draw_to_bytes(app, &self.theme)?
        };
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
    }

    fn handle_render(&mut self, params: RenderParams, candidates_tsv: &[u8]) -> Response {
        let RenderParams {
            prefix,
            cursor_row,
            cursor_col,
            term_cols,
            term_rows,
            selected,
        } = params;
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

        if app.filtered_indices.is_empty() {
            debug!("render request had no matching candidates after filter");
            self.fuzzy = Some(app.take_fuzzy());
            return Response::Empty;
        }

        let scroll_bytes = cap_viewport_and_scroll(&mut app, term_rows);

        if app.max_visible == 0 {
            debug!("render request had zero visible rows");
            self.fuzzy = Some(app.take_fuzzy());
            return Response::Empty;
        }

        if let Some(idx) = selected {
            app.set_selected(idx);
        }

        // Read cursor_row before render to avoid borrowing app after render
        let cursor_row_final = app.cursor_row;
        let filtered_count = app.filtered_indices.len();
        let selected_original_idx = app.selected_original_idx();
        let result = ui::render::render_popup_to_bytes(&app, &self.theme);

        match result {
            Ok((mut tty_bytes, popup)) => {
                let reuse_token = compute_reuse_token(&app.prefix, tsv_str, &app, &popup);
                self.fuzzy = Some(app.take_fuzzy());
                if !scroll_bytes.is_empty() {
                    let mut combined = scroll_bytes;
                    combined.append(&mut tty_bytes);
                    tty_bytes = combined;
                }
                debug!(
                    popup_row = popup.row,
                    popup_height = popup.height,
                    cursor_row = cursor_row_final,
                    reuse_token,
                    tty_bytes = tty_bytes.len(),
                    "render complete"
                );
                let metadata = popup.format_metadata(
                    cursor_row_final,
                    reuse_token,
                    filtered_count,
                    selected_original_idx,
                );
                Response::Success {
                    tty_bytes,
                    metadata: Some(metadata),
                }
            }
            Err(e) => {
                self.fuzzy = Some(app.take_fuzzy());
                error!(error = %e, "render failed");
                Response::Error(format!("render failed: {}", e))
            }
        }
    }

    /// Common session initialization for popup sessions.
    ///
    /// Parses candidates, creates `App`, filters, caps viewport, and calls `select_first()`.
    /// Returns `None` if an early exit was needed (empty candidates, no matches, etc.),
    /// in which case a `DONE 1` response has already been sent.
    #[allow(clippy::too_many_arguments)]
    fn setup_session<W: Write>(
        &mut self,
        writer: &mut W,
        prefix: String,
        cursor_row: u16,
        cursor_col: u16,
        term_cols: u16,
        term_rows: u16,
        tsv: &str,
    ) -> Option<(App, Vec<u8>)> {
        let candidates: Vec<Candidate> = tsv
            .lines()
            .filter(|line| !line.is_empty())
            .map(Candidate::parse_line)
            .collect();

        if candidates.is_empty() {
            let _ = writeln!(writer, "DONE 1 ");
            let _ = writer.flush();
            return None;
        }

        let fuzzy = self.fuzzy.take().unwrap_or_default();
        let mut app = App::new_with_matcher(
            candidates, prefix, cursor_row, cursor_col, term_cols, term_rows, fuzzy,
        );

        if app.filtered_indices.is_empty() {
            self.fuzzy = Some(app.take_fuzzy());
            let _ = writeln!(writer, "DONE 1 ");
            let _ = writer.flush();
            return None;
        }

        let scroll_bytes = cap_viewport_and_scroll(&mut app, term_rows);

        if app.max_visible == 0 {
            self.fuzzy = Some(app.take_fuzzy());
            let _ = writeln!(writer, "DONE 1 ");
            let _ = writer.flush();
            return None;
        }

        app.select_first();
        Some((app, scroll_bytes))
    }

    fn handle_complete<R: BufRead, W: Write>(
        &mut self,
        reader: &mut R,
        writer: &mut W,
        params: CompleteParams,
        tsv: &str,
    ) {
        let CompleteParams {
            prefix,
            cursor_row,
            cursor_col,
            term_cols,
            term_rows,
            reuse_popup,
            shift_tab_sequence,
        } = params;

        let (mut app, scroll_bytes) = match self.setup_session(
            writer, prefix, cursor_row, cursor_col, term_cols, term_rows, tsv,
        ) {
            Some(v) => v,
            None => return,
        };

        let reuse_fast_path = reuse_popup && scroll_bytes.is_empty();
        if self
            .send_frame(writer, &app, &scroll_bytes, reuse_fast_path)
            .is_err()
        {
            self.fuzzy = Some(app.take_fuzzy());
            return;
        }

        loop {
            let mut msg_line = String::new();
            if reader.read_line(&mut msg_line).is_err() || msg_line.is_empty() {
                break; // Connection closed
            }
            let msg_line = msg_line.trim_end();

            if let Some(len_str) = msg_line.strip_prefix("KEY ") {
                let byte_count: usize = len_str.parse().unwrap_or(0);
                if byte_count == 0 {
                    let _ = writeln!(writer, "NONE");
                    let _ = writer.flush();
                    continue;
                }
                if byte_count > MAX_INLINE_KEY_BYTES {
                    if drain_key_payload(reader, byte_count).is_err() {
                        break;
                    }
                    let _ = writeln!(writer, "DONE 3 {}", app.filter_text);
                    let _ = writer.flush();
                    break;
                }
                let mut key_buf = vec![0u8; byte_count];
                if std::io::Read::read_exact(reader, &mut key_buf).is_err() {
                    break;
                }

                let action = input::parse_tty_bytes_with_shift_tab(
                    &key_buf,
                    &self.key_bindings,
                    shift_tab_sequence.as_deref(),
                )
                .unwrap_or(Action::None);

                match action {
                    Action::MoveDown | Action::MoveUp | Action::PageDown | Action::PageUp => {
                        apply_navigation(&mut app, action);
                        if self.send_frame(writer, &app, &[], false).is_err() {
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
                        if self.send_frame(writer, &app, &clear_bytes, false).is_err() {
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
                        if self.send_frame(writer, &app, &clear_bytes, false).is_err() {
                            break;
                        }
                    }
                    Action::Confirm => {
                        match app.selected_candidate() {
                            Some(c) => {
                                let _ = writeln!(writer, "DONE 0 {}", c.text_with_suffix());
                            }
                            None => {
                                let _ = writeln!(writer, "DONE 1 {}", app.filter_text);
                            }
                        }
                        let _ = writer.flush();
                        break;
                    }
                    Action::DismissWithSpace => {
                        match app.selected_candidate() {
                            Some(c) => {
                                let _ =
                                    writeln!(writer, "DONE 2 {}", c.text_for_dismiss_with_space());
                            }
                            None => {
                                let _ = writeln!(writer, "DONE 2 {} ", app.filter_text);
                            }
                        }
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
                    Action::Resize(_, _) => {
                        let _ = writeln!(writer, "NONE");
                        let _ = writer.flush();
                    }
                    Action::None => {
                        let _ = writeln!(writer, "DONE 3 {}", app.filter_text);
                        let _ = writer.flush();
                        break;
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

fn drain_key_payload<R: std::io::Read>(reader: &mut R, byte_count: usize) -> io::Result<()> {
    let mut remaining = byte_count;
    let mut buf = [0u8; 256];

    while remaining > 0 {
        let chunk_len = remaining.min(buf.len());
        let read = reader.read(&mut buf[..chunk_len])?;
        if read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "short KEY payload while draining passthrough bytes",
            ));
        }
        remaining -= read;
    }

    Ok(())
}

fn parse_terminal_dims(parts: &[&str]) -> (u16, u16, u16, u16) {
    let cursor_row: u16 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let cursor_col: u16 = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    let term_cols: u16 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(80);
    let term_rows: u16 = parts.get(4).and_then(|s| s.parse().ok()).unwrap_or(24);
    (cursor_row, cursor_col, term_cols, term_rows)
}

fn read_prefix_and_tsv(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    command: &str,
) -> Result<(String, String), ()> {
    let prefix = match read_text_line(reader) {
        Ok(p) => p,
        Err(_) => {
            warn!("invalid text {} prefix", command);
            let _ = writeln!(writer, "ERROR invalid prefix");
            let _ = writer.flush();
            return Err(());
        }
    };
    let tsv = match read_tsv_payload(reader) {
        Ok(t) => t,
        Err(msg) => {
            let _ = writeln!(writer, "ERROR {}", msg);
            let _ = writer.flush();
            return Err(());
        }
    };
    Ok((prefix, tsv))
}

/// Cap `app.max_visible` to fit within `term_rows`, then compute scroll-up
/// bytes needed to make room for the popup below the cursor.
fn apply_navigation(app: &mut App, action: Action) {
    match action {
        Action::MoveDown => app.move_down(),
        Action::MoveUp => app.move_up(),
        Action::PageDown => app.page_down(),
        Action::PageUp => app.page_up(),
        _ => {}
    }
}

fn cap_viewport_and_scroll(app: &mut App, term_rows: u16) -> Vec<u8> {
    let max_popup_height = term_rows.saturating_sub(1);
    if app.max_visible as u16 + 2 > max_popup_height {
        app.max_visible = max_popup_height.saturating_sub(2).max(1) as usize;
    }

    if app.max_visible == 0 {
        return Vec::new();
    }

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
    scroll_bytes
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

#[cfg(test)]
mod tests {
    use super::{CompleteParams, DaemonServer, RenderParams, read_text_line};
    use crate::config::Config;
    use crate::fuzzy::FuzzyMatcher;
    use std::io::{BufRead, BufReader, Cursor, Read, Write};
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;
    use std::thread;

    fn read_frame(reader: &mut BufReader<UnixStream>) -> (String, String) {
        let mut header = String::new();
        reader.read_line(&mut header).unwrap();
        assert!(header.starts_with("FRAME "), "header was: {header:?}");

        let tty_len = header
            .split_whitespace()
            .last()
            .and_then(|token| token.parse::<usize>().ok())
            .expect("tty_len in frame header");
        let mut tty_bytes = vec![0; tty_len];
        reader.read_exact(&mut tty_bytes).unwrap();
        (header, String::from_utf8_lossy(&tty_bytes).into_owned())
    }

    fn send_key(writer: &mut UnixStream, bytes: &[u8]) {
        writeln!(writer, "KEY {}", bytes.len()).unwrap();
        writer.write_all(bytes).unwrap();
        writer.flush().unwrap();
    }

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

    fn assert_passthrough_key(prefix: &str, key: &[u8], expected_done: &str) {
        let (server_stream, client_stream) = UnixStream::pair().unwrap();
        let prefix = prefix.to_string();
        let handle = thread::spawn(move || {
            let mut server = test_server();
            let mut reader = BufReader::new(&server_stream);
            let mut writer = std::io::BufWriter::new(&server_stream);
            server.handle_complete(
                &mut reader,
                &mut writer,
                CompleteParams {
                    prefix,
                    cursor_row: 5,
                    cursor_col: 2,
                    term_cols: 80,
                    term_rows: 24,
                    reuse_popup: false,
                    shift_tab_sequence: None,
                },
                "git\tcommand\tcommand\ngizmo\tcommand\tcommand\n",
            );
        });

        let mut writer = client_stream.try_clone().unwrap();
        let mut reader = BufReader::new(client_stream);

        let _ = read_frame(&mut reader);
        send_key(&mut writer, key);

        let mut done = String::new();
        reader.read_line(&mut done).unwrap();
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), expected_done);

        drop(reader);
        drop(writer);
        handle.join().unwrap();
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
    fn handle_render_empty_after_filter_returns_empty() {
        let mut server = test_server();
        // Candidates exist but prefix "zzz" matches none after fuzzy filter
        let response = server.handle_render(
            RenderParams {
                prefix: "zzz".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                selected: None,
            },
            b"git\tcommand\tcommand\ngrep\tcommand\tcommand\n",
        );

        assert!(
            matches!(response, crate::protocol::Response::Empty),
            "expected Empty when no candidates match filter"
        );
        assert!(server.fuzzy.is_some());
    }

    #[test]
    fn handle_render_includes_reuse_token_metadata() {
        let mut server = test_server();
        let response = server.handle_render(
            RenderParams {
                prefix: "gi".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                selected: None,
            },
            b"git\tcommand\tcommand\n",
        );

        match response {
            crate::protocol::Response::Success { metadata, .. } => {
                let metadata = metadata.unwrap();
                assert!(metadata.contains("reuse_token="));
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    #[test]
    fn handle_complete_sends_initial_frame_for_new_popup() {
        let mut server = test_server();
        let (server_stream, client_stream) = UnixStream::pair().unwrap();
        let handle = thread::spawn(move || {
            let mut reader = BufReader::new(&server_stream);
            let mut writer = std::io::BufWriter::new(&server_stream);
            server.handle_complete(
                &mut reader,
                &mut writer,
                CompleteParams {
                    prefix: "gi".to_string(),
                    cursor_row: 5,
                    cursor_col: 2,
                    term_cols: 80,
                    term_rows: 24,
                    reuse_popup: false,
                    shift_tab_sequence: None,
                },
                "git\tcommand\tcommand\ngizmo\tcommand\tcommand\n",
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

    #[test]
    fn handle_complete_reuse_popup_redraws_popup_without_filter_line() {
        let mut server = test_server();
        let (server_stream, client_stream) = UnixStream::pair().unwrap();
        let handle = thread::spawn(move || {
            let mut reader = BufReader::new(&server_stream);
            let mut writer = std::io::BufWriter::new(&server_stream);
            server.handle_complete(
                &mut reader,
                &mut writer,
                CompleteParams {
                    prefix: "gi".to_string(),
                    cursor_row: 5,
                    cursor_col: 2,
                    term_cols: 80,
                    term_rows: 24,
                    reuse_popup: true,
                    shift_tab_sequence: None,
                },
                "git\tcommand\tcommand\ngizmo\tcommand\tcommand\n",
            );
        });

        let mut reader = BufReader::new(client_stream);
        let (header, tty) = read_frame(&mut reader);
        assert!(header.starts_with("FRAME "));
        assert!(tty.contains("┌"));
        assert!(!tty.contains("\u{1b}[6;1Hgi"));

        drop(reader);
        handle.join().unwrap();
    }

    #[test]
    fn handle_complete_initial_frame_includes_popup_border() {
        let (server_stream, client_stream) = UnixStream::pair().unwrap();
        let handle = thread::spawn(move || {
            let mut server = test_server();
            let mut reader = BufReader::new(&server_stream);
            let mut writer = std::io::BufWriter::new(&server_stream);
            server.handle_complete(
                &mut reader,
                &mut writer,
                CompleteParams {
                    prefix: "gi".to_string(),
                    cursor_row: 5,
                    cursor_col: 2,
                    term_cols: 80,
                    term_rows: 24,
                    reuse_popup: false,
                    shift_tab_sequence: None,
                },
                "git\tcommand\tcommand\n",
            );
        });

        let mut reader = BufReader::new(&client_stream);
        let mut header = String::new();
        reader.read_line(&mut header).unwrap();
        assert!(header.starts_with("FRAME "));

        let tty_len = header
            .split_whitespace()
            .last()
            .and_then(|token| token.parse::<usize>().ok())
            .expect("tty_len in frame header");
        let mut tty_bytes = vec![0; tty_len];
        reader.read_exact(&mut tty_bytes).unwrap();
        let tty = String::from_utf8_lossy(&tty_bytes);
        assert!(tty.contains("┌"));
        assert!(tty.contains("git"));

        drop(reader);
        drop(client_stream);
        handle.join().unwrap();
    }

    #[test]
    fn handle_complete_tab_after_typing_selects_top_filtered_candidate() {
        let (server_stream, client_stream) = UnixStream::pair().unwrap();
        let handle = thread::spawn(move || {
            let mut server = test_server();
            let mut reader = BufReader::new(&server_stream);
            let mut writer = std::io::BufWriter::new(&server_stream);
            server.handle_complete(
                &mut reader,
                &mut writer,
                CompleteParams {
                    prefix: "".to_string(),
                    cursor_row: 5,
                    cursor_col: 2,
                    term_cols: 80,
                    term_rows: 24,
                    reuse_popup: false,
                    shift_tab_sequence: None,
                },
                "ab\tcommand\tcommand\nax\tcommand\tcommand\nb\tcommand\tcommand\n",
            );
        });

        let mut writer = client_stream.try_clone().unwrap();
        let mut reader = BufReader::new(client_stream);

        let _ = read_frame(&mut reader);
        send_key(&mut writer, b"a");
        let _ = read_frame(&mut reader);

        send_key(&mut writer, b"\t");
        let _ = read_frame(&mut reader);

        send_key(&mut writer, b"\r");
        let mut done = String::new();
        reader.read_line(&mut done).unwrap();
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), "DONE 0 ab ");

        drop(reader);
        drop(writer);
        handle.join().unwrap();
    }

    #[test]
    fn handle_complete_confirm_after_typing_returns_filter_text() {
        let (server_stream, client_stream) = UnixStream::pair().unwrap();
        let handle = thread::spawn(move || {
            let mut server = test_server();
            let mut reader = BufReader::new(&server_stream);
            let mut writer = std::io::BufWriter::new(&server_stream);
            server.handle_complete(
                &mut reader,
                &mut writer,
                CompleteParams {
                    prefix: "".to_string(),
                    cursor_row: 5,
                    cursor_col: 2,
                    term_cols: 80,
                    term_rows: 24,
                    reuse_popup: false,
                    shift_tab_sequence: None,
                },
                "ab\tcommand\tcommand\nax\tcommand\tcommand\nb\tcommand\tcommand\n",
            );
        });

        let mut writer = client_stream.try_clone().unwrap();
        let mut reader = BufReader::new(client_stream);

        let _ = read_frame(&mut reader);
        send_key(&mut writer, b"a");
        let _ = read_frame(&mut reader);

        send_key(&mut writer, b"\r");
        let mut done = String::new();
        reader.read_line(&mut done).unwrap();
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), "DONE 1 a");

        drop(reader);
        drop(writer);
        handle.join().unwrap();
    }

    #[test]
    fn handle_complete_utf8_key_updates_filter_instead_of_passthrough() {
        let mut server = test_server();
        let mut input = Vec::new();
        writeln!(&mut input, "KEY {}", "あ".len()).unwrap();
        input.extend_from_slice("あ".as_bytes());

        let mut reader = BufReader::new(Cursor::new(input));
        let mut writer = Vec::new();
        server.handle_complete(
            &mut reader,
            &mut writer,
            CompleteParams {
                prefix: "".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "git\tcommand\tcommand\ngrep\tcommand\tcommand\n",
        );

        let output = String::from_utf8_lossy(&writer);
        assert!(output.contains("DONE 1 "), "output was: {output}");
        assert!(output.contains('あ'), "output was: {output}");
        assert!(!output.contains("DONE 3"), "output was: {output}");
    }

    #[test]
    fn handle_complete_space_after_selection_returns_selected_candidate() {
        let (server_stream, client_stream) = UnixStream::pair().unwrap();
        let handle = thread::spawn(move || {
            let mut server = test_server();
            let mut reader = BufReader::new(&server_stream);
            let mut writer = std::io::BufWriter::new(&server_stream);
            server.handle_complete(
                &mut reader,
                &mut writer,
                CompleteParams {
                    prefix: "".to_string(),
                    cursor_row: 5,
                    cursor_col: 2,
                    term_cols: 80,
                    term_rows: 24,
                    reuse_popup: false,
                    shift_tab_sequence: None,
                },
                "ab\tcommand\tcommand\nax\tcommand\tcommand\nb\tcommand\tcommand\n",
            );
        });

        let mut writer = client_stream.try_clone().unwrap();
        let mut reader = BufReader::new(client_stream);

        let _ = read_frame(&mut reader);
        send_key(&mut writer, b"a");
        let _ = read_frame(&mut reader);

        send_key(&mut writer, b"\t");
        let _ = read_frame(&mut reader);

        send_key(&mut writer, b" ");
        let mut done = String::new();
        reader.read_line(&mut done).unwrap();
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), "DONE 2 ab ");

        drop(reader);
        drop(writer);
        handle.join().unwrap();
    }

    #[test]
    fn handle_complete_space_after_empty_kind_selection_appends_space() {
        let (server_stream, client_stream) = UnixStream::pair().unwrap();
        let handle = thread::spawn(move || {
            let mut server = test_server();
            let mut reader = BufReader::new(&server_stream);
            let mut writer = std::io::BufWriter::new(&server_stream);
            server.handle_complete(
                &mut reader,
                &mut writer,
                CompleteParams {
                    prefix: "gi".to_string(),
                    cursor_row: 5,
                    cursor_col: 2,
                    term_cols: 80,
                    term_rows: 24,
                    reuse_popup: false,
                    shift_tab_sequence: None,
                },
                "git\tcommand\t\ngizmo\tcommand\t\n",
            );
        });

        let mut writer = client_stream.try_clone().unwrap();
        let mut reader = BufReader::new(client_stream);

        let _ = read_frame(&mut reader);
        send_key(&mut writer, b" ");
        let mut done = String::new();
        reader.read_line(&mut done).unwrap();
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), "DONE 2 git ");

        drop(reader);
        drop(writer);
        handle.join().unwrap();
    }

    #[test]
    fn handle_complete_unknown_key_passthroughs_filter_text() {
        assert_passthrough_key("gi", b"\x1b[D", "DONE 3 gi");
    }

    #[test]
    fn handle_complete_ctrl_bindings_passthrough_filter_text() {
        for key in [b"\x01".as_slice(), b"\x05", b"\x0b"] {
            assert_passthrough_key("gi", key, "DONE 3 gi");
        }
    }

    #[test]
    fn handle_complete_ctrl_j_passthrough_does_not_inject_newline() {
        let (server_stream, client_stream) = UnixStream::pair().unwrap();
        let handle = thread::spawn(move || {
            let mut server = test_server();
            let mut reader = BufReader::new(&server_stream);
            let mut writer = std::io::BufWriter::new(&server_stream);
            server.handle_complete(
                &mut reader,
                &mut writer,
                CompleteParams {
                    prefix: "gi".to_string(),
                    cursor_row: 5,
                    cursor_col: 2,
                    term_cols: 80,
                    term_rows: 24,
                    reuse_popup: false,
                    shift_tab_sequence: None,
                },
                "git\tcommand\tcommand\ngizmo\tcommand\tcommand\n",
            );
        });

        let mut writer = client_stream.try_clone().unwrap();
        let mut reader = BufReader::new(client_stream);

        let _ = read_frame(&mut reader);
        send_key(&mut writer, b"\n");

        let mut done = String::new();
        reader.read_line(&mut done).unwrap();
        assert_eq!(done, "DONE 3 gi\n");

        let mut extra = String::new();
        reader.read_line(&mut extra).unwrap();
        assert!(
            extra.is_empty(),
            "unexpected extra protocol line: {extra:?}"
        );

        drop(reader);
        drop(writer);
        handle.join().unwrap();
    }

    #[test]
    fn handle_complete_oversized_key_drains_payload_and_passthroughs() {
        assert_passthrough_key("gi", b"\x1b[200~git status --short\x1b[201~", "DONE 3 gi");
    }

    fn assert_immediate_confirm(prefix: &str, candidates_tsv: &str, expected_done: &str) {
        let (server_stream, client_stream) = UnixStream::pair().unwrap();
        let prefix = prefix.to_string();
        let candidates_tsv = candidates_tsv.to_string();
        let handle = thread::spawn(move || {
            let mut server = test_server();
            let mut reader = BufReader::new(&server_stream);
            let mut writer = std::io::BufWriter::new(&server_stream);
            server.handle_complete(
                &mut reader,
                &mut writer,
                CompleteParams {
                    prefix,
                    cursor_row: 5,
                    cursor_col: 2,
                    term_cols: 80,
                    term_rows: 24,
                    reuse_popup: false,
                    shift_tab_sequence: None,
                },
                &candidates_tsv,
            );
        });

        let mut writer = client_stream.try_clone().unwrap();
        let mut reader = BufReader::new(client_stream);

        let _ = read_frame(&mut reader);
        send_key(&mut writer, b"\r");
        let mut done = String::new();
        reader.read_line(&mut done).unwrap();
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), expected_done);

        drop(reader);
        drop(writer);
        handle.join().unwrap();
    }

    #[test]
    fn handle_complete_confirm_with_common_prefix_selects_first() {
        assert_immediate_confirm(
            "fo",
            "foobar\tcommand\tcommand\nfoobaz\tcommand\tcommand\n",
            "DONE 0 foobar ",
        );
    }

    #[test]
    fn handle_complete_auto_selects_when_lcp_exceeds_prefix() {
        assert_immediate_confirm(
            "car",
            "cargo\tcommand\tcommand\ncargo-add\tcommand\tcommand\n",
            "DONE 0 cargo ",
        );
    }
}
