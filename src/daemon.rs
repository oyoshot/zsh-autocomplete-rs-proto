use std::collections::HashMap;
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
use crate::protocol::{
    self, Request, Response, TextClearRequest, TextCompleteRequest, TextCompleteResult,
    TextFrameHeader, TextRenderRequest, TextRequest, TextSessionRequest,
};
use crate::ui;
use tracing::{debug, error, info, info_span, warn};
use tracing_subscriber::EnvFilter;

const MAX_INLINE_KEY_BYTES: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ApplyResult {
    code: u8,
    text: String,
    chain: bool,
    execute: bool,
    restore_text: String,
}

impl ApplyResult {
    fn apply_only(text: String) -> Self {
        Self {
            chain: should_chain_after_apply(&text),
            text,
            code: 0,
            execute: false,
            restore_text: String::new(),
        }
    }

    fn confirm(text: String) -> Self {
        Self {
            chain: should_chain_after_apply(&text),
            text,
            code: 0,
            execute: true,
            restore_text: String::new(),
        }
    }

    fn cancel(text: String) -> Self {
        Self {
            code: 1,
            text,
            chain: false,
            execute: false,
            restore_text: String::new(),
        }
    }

    fn dismiss_with_space(text: String) -> Self {
        Self {
            chain: should_chain_after_apply(&text),
            text,
            code: 2,
            execute: false,
            restore_text: String::new(),
        }
    }
}

fn should_chain_after_apply(text: &str) -> bool {
    text.ends_with([' ', '/'])
}

fn encode_hex_bytes(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

fn write_apply_result<W: Write>(writer: &mut W, result: &ApplyResult) -> io::Result<()> {
    TextCompleteResult {
        code: result.code,
        text: result.text.clone(),
        chain: result.chain,
        execute: result.execute,
        restore_text: result.restore_text.clone(),
    }
    .write_to(writer)
}

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
    prev_popup_row: Option<u16>,
    prev_popup_height: Option<u16>,
    command_position: bool,
    accept_single: bool,
    reuse_popup: bool,
    shift_tab_sequence: Option<Vec<u8>>,
}

/// Maximum number of candidate-set cache entries retained across connections.
/// When this limit is reached the oldest entry is evicted to bound memory use.
const CANDIDATE_CACHE_MAX_ENTRIES: usize = 8;
const ACTIVE_POPUP_MAX_ENTRIES: usize = 16;

#[derive(Clone)]
struct CandidateCacheEntry {
    source_prefix: String,
    tsv: String,
}

#[derive(Clone)]
struct ActivePopupEntry {
    prefix: String,
    tsv: String,
}

struct DaemonServer {
    config: Config,
    theme: Theme,
    key_bindings: KeyBindings,
    config_mtime: Option<SystemTime>,
    socket_path: PathBuf,
    fuzzy: Option<FuzzyMatcher>,
    /// Candidate-set cache keyed by `context_key` supplied by the shell.
    /// Each entry also records the prefix used to gather it so cache-only
    /// reuse can reject lookups that would broaden a compsys-prefiltered set.
    candidate_cache: HashMap<String, CandidateCacheEntry>,
    /// Insertion-order tracking for LRU eviction.
    candidate_cache_order: Vec<String>,
    /// Most recent auto-popup payload per shell session.
    active_popups: HashMap<String, ActivePopupEntry>,
    active_popup_order: Vec<String>,
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
            candidate_cache: HashMap::new(),
            candidate_cache_order: Vec::new(),
            active_popups: HashMap::new(),
            active_popup_order: Vec::new(),
        }
    }
}

/// Run a single complete session over arbitrary Read/Write streams.
///
/// Used by the subprocess fallback path where the shell communicates
/// via stdin/stdout using the same text protocol as the daemon.
#[allow(clippy::too_many_arguments)]
pub fn run_stdio_complete<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    prefix: String,
    cursor_row: u16,
    cursor_col: u16,
    term_cols: u16,
    term_rows: u16,
    command_position: bool,
    accept_single: bool,
    shift_tab_sequence: Option<Vec<u8>>,
    prev_popup_row: Option<u16>,
    prev_popup_height: Option<u16>,
    tsv: &str,
) {
    let config = Config::load();
    let theme = config.theme();
    let key_bindings = config.key_bindings();
    let mut server = DaemonServer {
        config,
        theme,
        key_bindings,
        config_mtime: None,
        socket_path: PathBuf::new(),
        fuzzy: Some(FuzzyMatcher::new()),
        candidate_cache: HashMap::new(),
        candidate_cache_order: Vec::new(),
        active_popups: HashMap::new(),
        active_popup_order: Vec::new(),
    };
    let params = CompleteParams {
        prefix,
        cursor_row,
        cursor_col,
        term_cols,
        term_rows,
        prev_popup_row,
        prev_popup_height,
        command_position,
        accept_single,
        reuse_popup: false,
        shift_tab_sequence,
    };
    server.handle_complete(reader, writer, params, tsv);
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
    fn touch_cached_key(&mut self, context_key: &str) {
        if self.candidate_cache.contains_key(context_key) {
            self.candidate_cache_order.retain(|k| k != context_key);
            self.candidate_cache_order.push(context_key.to_string());
        }
    }

    fn touch_active_popup_key(&mut self, popup_key: &str) {
        if self.active_popups.contains_key(popup_key) {
            self.active_popup_order.retain(|k| k != popup_key);
            self.active_popup_order.push(popup_key.to_string());
        }
    }

    /// Look up a cached TSV payload by `context_key`.
    /// Returns `Some(tsv)` only when `current_prefix` exactly matches the
    /// cached `source_prefix`; otherwise the lookup is treated as a miss
    /// because the cached set may already be narrowed by compsys.
    fn get_cached_tsv(&mut self, context_key: &str, current_prefix: &str) -> Option<String> {
        let entry = self.candidate_cache.get(context_key)?.clone();
        if current_prefix != entry.source_prefix {
            return None;
        }
        self.touch_cached_key(context_key);
        Some(entry.tsv)
    }

    /// Store `tsv` in the candidate cache under `context_key`.
    /// Evicts the oldest entry when `CANDIDATE_CACHE_MAX_ENTRIES` is exceeded.
    /// Re-inserting an existing key moves it to the most-recent position.
    fn store_cached_tsv(&mut self, context_key: &str, source_prefix: String, tsv: String) {
        let entry = CandidateCacheEntry { source_prefix, tsv };
        if self.candidate_cache.contains_key(context_key) {
            self.candidate_cache.insert(context_key.to_string(), entry);
            self.touch_cached_key(context_key);
            return;
        }
        if self.candidate_cache_order.len() >= CANDIDATE_CACHE_MAX_ENTRIES {
            if let Some(oldest) = self.candidate_cache_order.first().cloned() {
                self.candidate_cache.remove(&oldest);
                self.candidate_cache_order.remove(0);
            }
        }
        self.candidate_cache_order.push(context_key.to_string());
        self.candidate_cache.insert(context_key.to_string(), entry);
    }

    fn get_active_popup(&mut self, popup_key: &str) -> Option<ActivePopupEntry> {
        let entry = self.active_popups.get(popup_key)?.clone();
        self.touch_active_popup_key(popup_key);
        Some(entry)
    }

    fn store_active_popup(&mut self, popup_key: &str, prefix: String, tsv: String) {
        let entry = ActivePopupEntry { prefix, tsv };
        if self.active_popups.contains_key(popup_key) {
            self.active_popups.insert(popup_key.to_string(), entry);
            self.touch_active_popup_key(popup_key);
            return;
        }
        if self.active_popup_order.len() >= ACTIVE_POPUP_MAX_ENTRIES {
            if let Some(oldest) = self.active_popup_order.first().cloned() {
                self.active_popups.remove(&oldest);
                self.active_popup_order.remove(0);
            }
        }
        self.active_popup_order.push(popup_key.to_string());
        self.active_popups.insert(popup_key.to_string(), entry);
    }

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
                        prefix: prefix.clone(),
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
    ///   APPLY chain=<0|1> execute=<0|1> restore_hex=<hex>\n
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
        let Some(request) = TextRequest::parse_header(header) else {
            warn!(header = header, "unknown text request");
            return false;
        };

        let mut writer = io::BufWriter::new(stream);

        match request {
            TextRequest::Render(TextRenderRequest {
                cursor_row,
                cursor_col,
                term_cols,
                term_rows,
                selected,
                context_key,
                popup_key,
            }) => {
                let (mut prefix, tsv_opt) =
                    match read_prefix_and_candidates(reader, &mut writer, "render") {
                        Ok(v) => v,
                        Err(()) => return false,
                    };

                // Cache-only request: TSV absent, context_key present.
                if tsv_opt.is_none() {
                    let active_popup = if context_key.is_none() {
                        popup_key
                            .as_deref()
                            .and_then(|key| self.get_active_popup(key))
                    } else {
                        None
                    };
                    if let Some(active_popup) = active_popup {
                        prefix = active_popup.prefix;
                        let cached_tsv = active_popup.tsv;
                        let _span = info_span!(
                            "render",
                            protocol = "text",
                            prefix_len = prefix.len(),
                            cursor_row,
                            cursor_col,
                            term_cols,
                            term_rows,
                            ?selected,
                            cache = "popup",
                            payload_bytes = cached_tsv.len()
                        )
                        .entered();
                        let response = self.handle_render(
                            RenderParams {
                                prefix: prefix.clone(),
                                cursor_row,
                                cursor_col,
                                term_cols,
                                term_rows,
                                selected,
                            },
                            cached_tsv.as_bytes(),
                        );
                        match response {
                            Response::Success {
                                tty_bytes,
                                metadata,
                            } => {
                                if let Some(ref key) = popup_key {
                                    self.store_active_popup(key, prefix, cached_tsv.clone());
                                }
                                let meta = metadata.unwrap_or_default();
                                let _ =
                                    protocol::write_text_ok(&mut writer, &meta, tty_bytes.len());
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
                    } else if let Some(ref key) = context_key {
                        if let Some(cached_tsv) = self.get_cached_tsv(key, &prefix) {
                            let _span = info_span!(
                                "render",
                                protocol = "text",
                                prefix_len = prefix.len(),
                                cursor_row,
                                cursor_col,
                                term_cols,
                                term_rows,
                                ?selected,
                                cache = "hit",
                                payload_bytes = cached_tsv.len()
                            )
                            .entered();
                            let response = self.handle_render(
                                RenderParams {
                                    prefix: prefix.clone(),
                                    cursor_row,
                                    cursor_col,
                                    term_cols,
                                    term_rows,
                                    selected,
                                },
                                cached_tsv.as_bytes(),
                            );
                            match response {
                                Response::Success {
                                    tty_bytes,
                                    metadata,
                                } => {
                                    if let Some(ref key) = popup_key {
                                        self.store_active_popup(key, prefix, cached_tsv.clone());
                                    }
                                    let meta = metadata.unwrap_or_default();
                                    let _ = protocol::write_text_ok(
                                        &mut writer,
                                        &meta,
                                        tty_bytes.len(),
                                    );
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
                        } else {
                            debug!(context_key = key.as_str(), "render cache miss");
                            let _ = writeln!(writer, "CACHE_MISS");
                            let _ = writer.flush();
                        }
                    } else {
                        // No candidates and no context_key: nothing to render.
                        let _ = writeln!(writer, "EMPTY");
                        let _ = writer.flush();
                    }
                    return false;
                }

                let tsv = tsv_opt.unwrap();
                // Update cache when context_key is provided with fresh TSV.
                if let Some(ref key) = context_key {
                    self.store_cached_tsv(key, prefix.clone(), tsv.clone());
                }

                let _span = info_span!(
                    "render",
                    protocol = "text",
                    prefix_len = prefix.len(),
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                    ?selected,
                    cache = "miss",
                    payload_bytes = tsv.len()
                )
                .entered();
                let response = self.handle_render(
                    RenderParams {
                        prefix: prefix.clone(),
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
                        if let Some(ref key) = popup_key {
                            self.store_active_popup(key, prefix, tsv);
                        }
                        let meta = metadata.unwrap_or_default();
                        let _ = protocol::write_text_ok(&mut writer, &meta, tty_bytes.len());
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
            TextRequest::Complete(TextCompleteRequest {
                cursor_row,
                cursor_col,
                term_cols,
                term_rows,
                prev_popup,
                command_position,
                accept_single,
                reuse_token,
                shift_tab_sequence,
                context_key,
                popup_key,
            }) => {
                let (mut prefix, tsv_opt) =
                    match read_prefix_and_candidates(reader, &mut writer, "complete") {
                        Ok(v) => v,
                        Err(()) => return false,
                    };

                // Cache-only request: resolve TSV from cache or report miss.
                let tsv = match tsv_opt {
                    None => {
                        if context_key.is_none() {
                            if let Some(ref key) = popup_key {
                                match self.get_active_popup(key) {
                                    Some(active_popup) => {
                                        debug!(
                                            popup_key = key.as_str(),
                                            "complete popup cache hit"
                                        );
                                        prefix = active_popup.prefix;
                                        active_popup.tsv
                                    }
                                    None => {
                                        debug!(
                                            popup_key = key.as_str(),
                                            "complete popup cache miss"
                                        );
                                        let _ = writeln!(writer, "CACHE_MISS");
                                        let _ = writer.flush();
                                        return false;
                                    }
                                }
                            } else {
                                let _ = write_apply_result(
                                    &mut writer,
                                    &ApplyResult::cancel(String::new()),
                                );
                                return false;
                            }
                        } else if let Some(ref key) = context_key {
                            match self.get_cached_tsv(key, &prefix) {
                                Some(cached) => {
                                    debug!(context_key = key.as_str(), "complete cache hit");
                                    cached
                                }
                                None => {
                                    debug!(context_key = key.as_str(), "complete cache miss");
                                    let _ = writeln!(writer, "CACHE_MISS");
                                    let _ = writer.flush();
                                    return false;
                                }
                            }
                        } else if let Some(ref key) = popup_key {
                            match self.get_active_popup(key) {
                                Some(active_popup) => {
                                    debug!(popup_key = key.as_str(), "complete popup cache hit");
                                    prefix = active_popup.prefix;
                                    active_popup.tsv
                                }
                                None => {
                                    debug!(popup_key = key.as_str(), "complete popup cache miss");
                                    let _ = writeln!(writer, "CACHE_MISS");
                                    let _ = writer.flush();
                                    return false;
                                }
                            }
                        } else {
                            let _ = write_apply_result(
                                &mut writer,
                                &ApplyResult::cancel(String::new()),
                            );
                            return false;
                        }
                    }
                    Some(tsv) => {
                        if let Some(ref key) = context_key {
                            self.store_cached_tsv(key, prefix.clone(), tsv.clone());
                        }
                        tsv
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
                    reuse_popup = reuse_token.is_some(),
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
                        prev_popup_row: prev_popup.map(|(row, _)| row),
                        prev_popup_height: prev_popup.map(|(_, height)| height),
                        command_position,
                        accept_single,
                        reuse_popup: reuse_token.is_some(),
                        shift_tab_sequence,
                    },
                    &tsv,
                );
                false
            }
            TextRequest::Clear(TextClearRequest {
                popup_row,
                popup_height,
                cursor_row,
            }) => {
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
            TextRequest::Ping => {
                let _span = info_span!("ping", protocol = "text").entered();
                let _ = writeln!(writer, "OK");
                let _ = writer.flush();
                false
            }
            TextRequest::Shutdown => {
                let _span = info_span!("shutdown", protocol = "text").entered();
                let _ = writeln!(writer, "OK");
                let _ = writer.flush();
                true
            }
        }
    }

    fn send_frame<W: Write>(
        &self,
        writer: &mut W,
        app: &App,
        extra_prefix: &[u8],
        prev_popup: Option<(u16, u16)>,
        popup_only: bool,
        common_prefix: Option<&str>,
    ) -> io::Result<()> {
        let (tty_bytes, popup) = if popup_only {
            ui::render::render_popup_to_bytes(app, &self.theme)?
        } else {
            ui::render::draw_to_bytes(app, &self.theme)?
        };
        let stale_clear_bytes = prev_popup
            .map(|(row, height)| {
                ui::render::clear_stale_rows_to_bytes(row, height, popup.row, popup.height)
            })
            .transpose()?
            .unwrap_or_default();
        let total_len = stale_clear_bytes.len() + extra_prefix.len() + tty_bytes.len();
        TextFrameHeader {
            popup_row: popup.row,
            popup_height: popup.height,
            cursor_row: app.cursor_row,
            common_prefix: common_prefix
                .filter(|value| ui::popup::is_safe_prefix(value))
                .map(str::to_string),
            tty_len: total_len,
        }
        .write_to(writer)?;
        if !stale_clear_bytes.is_empty() {
            writer.write_all(&stale_clear_bytes)?;
        }
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

        if !self.config.auto_insert_unambiguous {
            app.reset_filter_to_prefix();
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
                // Render metadata is for popup position only; auto-insert is handled
                // exclusively via FRAME headers in the complete (interactive) path.
                let metadata = popup.format_metadata(
                    cursor_row_final,
                    reuse_token,
                    filtered_count,
                    selected_original_idx,
                    None,
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
            let _ = write_apply_result(writer, &ApplyResult::cancel(String::new()));
            return None;
        }

        let fuzzy = self.fuzzy.take().unwrap_or_default();
        let mut app = App::new_with_matcher(
            candidates, prefix, cursor_row, cursor_col, term_cols, term_rows, fuzzy,
        );

        // When auto_insert_unambiguous is disabled, keep filter_text at the typed
        // prefix so that cancel/passthrough paths never return an extended value.
        if !self.config.auto_insert_unambiguous {
            app.reset_filter_to_prefix();
        }

        if app.filtered_indices.is_empty() {
            self.fuzzy = Some(app.take_fuzzy());
            let _ = write_apply_result(writer, &ApplyResult::cancel(String::new()));
            return None;
        }

        let scroll_bytes = cap_viewport_and_scroll(&mut app, term_rows);

        if app.max_visible == 0 {
            self.fuzzy = Some(app.take_fuzzy());
            let _ = write_apply_result(writer, &ApplyResult::cancel(String::new()));
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
            prev_popup_row,
            prev_popup_height,
            command_position,
            accept_single,
            reuse_popup,
            shift_tab_sequence,
        } = params;

        let (mut app, scroll_bytes) = match self.setup_session(
            writer, prefix, cursor_row, cursor_col, term_cols, term_rows, tsv,
        ) {
            Some(v) => v,
            None => return,
        };

        let initial_common_prefix = self
            .config
            .auto_insert_unambiguous
            .then(|| app.unambiguous_prefix().map(str::to_string))
            .flatten();

        if accept_single && app.filtered_indices.len() == 1 {
            if let Some(candidate) = app.selected_candidate() {
                let _ = write_apply_result(
                    writer,
                    &ApplyResult::apply_only(candidate.text_with_suffix_for_command_position(
                        &self.config.suffixes,
                        command_position,
                    )),
                );
            } else {
                let _ = write_apply_result(writer, &ApplyResult::cancel(String::new()));
            }
            self.fuzzy = Some(app.take_fuzzy());
            return;
        }

        let reuse_fast_path = reuse_popup && scroll_bytes.is_empty();
        let prev_popup = prev_popup_row.zip(prev_popup_height);
        if self
            .send_frame(
                writer,
                &app,
                &scroll_bytes,
                prev_popup,
                reuse_fast_path,
                initial_common_prefix.as_deref(),
            )
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

            match TextSessionRequest::parse(msg_line) {
                Some(TextSessionRequest::Key { byte_count }) => {
                    if byte_count == 0 {
                        let _ = writeln!(writer, "NONE");
                        let _ = writer.flush();
                        continue;
                    }
                    if byte_count > MAX_INLINE_KEY_BYTES {
                        let drained = match drain_key_payload(reader, byte_count) {
                            Ok(bytes) => bytes,
                            Err(_) => break,
                        };
                        let _ = write_apply_result(
                            writer,
                            &ApplyResult {
                                code: 3,
                                text: encode_hex_bytes(&drained),
                                chain: false,
                                execute: false,
                                restore_text: app.filter_text.clone(),
                            },
                        );
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
                            if self
                                .send_frame(writer, &app, &[], None, false, None)
                                .is_err()
                            {
                                break;
                            }
                        }
                        Action::TypeChar(c) => {
                            let clear_bytes = ui::render::clear_to_bytes(&app).unwrap_or_default();
                            app.type_char(c);
                            if app.filtered_indices.is_empty() {
                                let _ = write_apply_result(
                                    writer,
                                    &ApplyResult::cancel(app.filter_text.clone()),
                                );
                                break;
                            }
                            if self
                                .send_frame(writer, &app, &clear_bytes, None, false, None)
                                .is_err()
                            {
                                break;
                            }
                        }
                        Action::Backspace => {
                            let clear_bytes = ui::render::clear_to_bytes(&app).unwrap_or_default();
                            if !app.backspace() {
                                let _ =
                                    write_apply_result(writer, &ApplyResult::cancel(String::new()));
                                break;
                            }
                            if app.filtered_indices.is_empty()
                                || app.filter_text.len() < app.prefix.len()
                            {
                                let _ = write_apply_result(
                                    writer,
                                    &ApplyResult::cancel(app.filter_text.clone()),
                                );
                                break;
                            }
                            if self
                                .send_frame(writer, &app, &clear_bytes, None, false, None)
                                .is_err()
                            {
                                break;
                            }
                        }
                        Action::Confirm => {
                            match app.selected_candidate() {
                                Some(c) => {
                                    let _ = write_apply_result(
                                        writer,
                                        &ApplyResult::confirm(
                                            c.text_with_suffix_for_command_position(
                                                &self.config.suffixes,
                                                command_position,
                                            ),
                                        ),
                                    );
                                }
                                None => {
                                    let _ = write_apply_result(
                                        writer,
                                        &ApplyResult::cancel(app.filter_text.clone()),
                                    );
                                }
                            }
                            break;
                        }
                        Action::DismissWithSpace => {
                            match app.selected_candidate() {
                                Some(c) => {
                                    let _ = write_apply_result(
                                        writer,
                                        &ApplyResult::dismiss_with_space(
                                            c.text_for_dismiss_with_space(
                                                &self.config.suffixes,
                                                command_position,
                                            ),
                                        ),
                                    );
                                }
                                None => {
                                    let _ = write_apply_result(
                                        writer,
                                        &ApplyResult::dismiss_with_space(format!(
                                            "{} ",
                                            app.filter_text
                                        )),
                                    );
                                }
                            }
                            break;
                        }
                        Action::Cancel => {
                            let text = if app.filter_text != app.prefix {
                                &app.filter_text
                            } else {
                                ""
                            };
                            let _ =
                                write_apply_result(writer, &ApplyResult::cancel(text.to_string()));
                            break;
                        }
                        Action::Resize(_, _) => {
                            let _ = writeln!(writer, "NONE");
                            let _ = writer.flush();
                        }
                        Action::None => {
                            let _ = write_apply_result(
                                writer,
                                &ApplyResult {
                                    code: 3,
                                    text: encode_hex_bytes(&key_buf),
                                    chain: false,
                                    execute: false,
                                    restore_text: app.filter_text.clone(),
                                },
                            );
                            break;
                        }
                    }
                }
                Some(TextSessionRequest::Resize {
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                }) => {
                    let previous_popup = crate::ui::popup::Popup::compute(&app);
                    app.set_terminal_state(cursor_row, cursor_col, term_cols, term_rows);
                    let scroll_bytes = cap_viewport_and_scroll(&mut app, term_rows);
                    let new_popup = crate::ui::popup::Popup::compute(&app);
                    let (extra_prefix, prev_popup) =
                        if resize_requires_full_popup_clear(&previous_popup, &new_popup) {
                            let mut clear_bytes = match ui::render::clear_rect_to_bytes(
                                previous_popup.row,
                                previous_popup.height,
                                app.cursor_row,
                            ) {
                                Ok(bytes) => bytes,
                                Err(_) => break,
                            };
                            clear_bytes.extend_from_slice(&scroll_bytes);
                            (clear_bytes, None)
                        } else {
                            (
                                scroll_bytes,
                                Some((previous_popup.row, previous_popup.height)),
                            )
                        };
                    if self
                        .send_frame(writer, &app, &extra_prefix, prev_popup, false, None)
                        .is_err()
                    {
                        break;
                    }
                }
                None => {
                    // Unknown message, ignore
                    let _ = writeln!(writer, "NONE");
                    let _ = writer.flush();
                }
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

fn drain_key_payload<R: std::io::Read>(reader: &mut R, byte_count: usize) -> io::Result<Vec<u8>> {
    let mut remaining = byte_count;
    let mut payload = Vec::with_capacity(byte_count);
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
        payload.extend_from_slice(&buf[..read]);
        remaining -= read;
    }

    Ok(payload)
}

/// Reads prefix line and optional TSV candidate payload from the text protocol stream.
///
/// Returns `(prefix, None)` when `END` follows the prefix line directly
/// (no TSV candidates), which signals a daemon cache lookup attempt.
/// Returns `(prefix, Some(tsv))` when TSV candidates are present.
fn read_prefix_and_candidates(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    command: &str,
) -> Result<(String, Option<String>), ()> {
    const MAX_TSV_BYTES: usize = 1_048_576;
    let prefix = match read_text_line(reader) {
        Ok(p) => p,
        Err(_) => {
            warn!("invalid text {} prefix", command);
            let _ = writeln!(writer, "ERROR invalid prefix");
            let _ = writer.flush();
            return Err(());
        }
    };

    // Peek at the first line of the payload to decide cache-only vs. full request.
    let mut first_line = String::new();
    if reader.read_line(&mut first_line).is_err() || first_line.is_empty() {
        let _ = writeln!(writer, "ERROR missing payload");
        let _ = writer.flush();
        return Err(());
    }

    if first_line.trim_end() == "END" {
        // Cache-only request: no candidates were sent.
        return Ok((prefix, None));
    }

    if first_line.len() > MAX_TSV_BYTES {
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
        return Err(());
    }

    // Full request: first_line is the start of the TSV; read the rest.
    let mut tsv = first_line;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() || line.is_empty() {
            let _ = writeln!(writer, "ERROR missing END");
            let _ = writer.flush();
            return Err(());
        }
        if line.trim_end() == "END" {
            return Ok((prefix, Some(tsv)));
        }
        if tsv.len() + line.len() > MAX_TSV_BYTES {
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
            return Err(());
        }
        tsv.push_str(&line);
    }
}

fn apply_navigation(app: &mut App, action: Action) {
    match action {
        Action::MoveDown => app.move_down(),
        Action::MoveUp => app.move_up(),
        Action::PageDown => app.page_down(),
        Action::PageUp => app.page_up(),
        _ => {}
    }
}

fn resize_requires_full_popup_clear(
    previous_popup: &crate::ui::popup::Popup,
    new_popup: &crate::ui::popup::Popup,
) -> bool {
    previous_popup.col != new_popup.col || previous_popup.width != new_popup.width
}

/// Cap `app.max_visible` to fit within `term_rows`, then compute scroll-up
/// bytes needed to make room for the popup below the cursor.
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

pub fn read_tsv_payload(reader: &mut impl BufRead) -> Result<String, String> {
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
    use super::{
        CompleteParams, DaemonServer, RenderParams, read_prefix_and_candidates, read_text_line,
    };
    use crate::config::Config;
    use crate::fuzzy::FuzzyMatcher;
    use crate::protocol::{TextCompleteRequest, TextCompleteResult, TextFrameHeader};
    use crate::ui;
    use std::collections::HashMap;
    use std::io::{BufRead, BufReader, Cursor, Read, Write};
    use std::net::Shutdown;
    use std::os::unix::net::UnixStream;
    use std::path::PathBuf;

    fn read_frame_bytes<R: BufRead>(reader: &mut R) -> (String, Vec<u8>) {
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
        (header, tty_bytes)
    }

    fn read_frame<R: BufRead>(reader: &mut R) -> (String, String) {
        let (header, tty_bytes) = read_frame_bytes(reader);
        (header, String::from_utf8_lossy(&tty_bytes).into_owned())
    }

    enum SessionMessage<'a> {
        Key(&'a [u8]),
        Resize {
            cursor_row: u16,
            cursor_col: u16,
            term_cols: u16,
            term_rows: u16,
        },
    }

    fn session_input(messages: &[SessionMessage<'_>]) -> Vec<u8> {
        let mut input = Vec::new();
        for message in messages {
            match message {
                SessionMessage::Key(bytes) => {
                    writeln!(&mut input, "KEY {}", bytes.len()).unwrap();
                    input.extend_from_slice(bytes);
                }
                SessionMessage::Resize {
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                } => {
                    writeln!(
                        &mut input,
                        "RESIZE {cursor_row} {cursor_col} {term_cols} {term_rows}"
                    )
                    .unwrap();
                }
            }
        }
        input
    }

    fn run_complete_session(
        server: &mut DaemonServer,
        params: CompleteParams,
        tsv: &str,
        messages: &[SessionMessage<'_>],
    ) -> BufReader<Cursor<Vec<u8>>> {
        let input = session_input(messages);
        let mut reader = BufReader::new(Cursor::new(input));
        let mut writer = Vec::new();
        server.handle_complete(&mut reader, &mut writer, params, tsv);
        BufReader::new(Cursor::new(writer))
    }

    fn read_done<R: BufRead>(reader: &mut R) -> (String, String) {
        let mut done = String::new();
        reader.read_line(&mut done).unwrap();
        let mut apply = String::new();
        reader.read_line(&mut apply).unwrap();
        (done, apply)
    }

    fn text_request_input(header: &str, prefix: &str, tsv: Option<&str>) -> Vec<u8> {
        let mut input = Vec::new();
        writeln!(&mut input, "{header}").unwrap();
        writeln!(&mut input, "{prefix}").unwrap();
        if let Some(tsv) = tsv {
            write!(&mut input, "{tsv}").unwrap();
        }
        writeln!(&mut input, "END").unwrap();
        input
    }

    fn run_text_request(server: &mut DaemonServer, input: Vec<u8>) -> Vec<u8> {
        let (mut client_stream, server_stream) = UnixStream::pair().unwrap();
        client_stream.write_all(&input).unwrap();
        client_stream.shutdown(Shutdown::Write).unwrap();

        {
            let mut reader = BufReader::new(&server_stream);
            server.handle_text_connection(&mut reader, &server_stream);
        }

        drop(server_stream);

        let mut output = Vec::new();
        client_stream.read_to_end(&mut output).unwrap();
        output
    }

    fn read_text_ok(bytes: &[u8]) -> (String, String) {
        let mut reader = BufReader::new(Cursor::new(bytes));
        let mut header = String::new();
        reader.read_line(&mut header).unwrap();
        assert!(header.starts_with("OK "), "header was: {header:?}");
        let tty_len = header
            .split_whitespace()
            .find_map(|token| token.strip_prefix("tty_len="))
            .and_then(|value| value.parse::<usize>().ok())
            .expect("tty_len in OK header");
        let mut tty_bytes = vec![0; tty_len];
        reader.read_exact(&mut tty_bytes).unwrap();
        (header, String::from_utf8_lossy(&tty_bytes).into_owned())
    }

    fn read_text_complete_result(bytes: &[u8]) -> TextCompleteResult {
        let mut reader = BufReader::new(Cursor::new(bytes));
        let mut done = String::new();
        reader.read_line(&mut done).unwrap();
        TextCompleteResult::read_from(&mut reader, done.trim_end()).unwrap()
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
            candidate_cache: HashMap::new(),
            candidate_cache_order: Vec::new(),
            active_popups: HashMap::new(),
            active_popup_order: Vec::new(),
        }
    }

    fn assert_passthrough_key(prefix: &str, key: &[u8], expected_done: &str) {
        let mut server = test_server();
        let mut reader = run_complete_session(
            &mut server,
            CompleteParams {
                prefix: prefix.to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "git\tcommand\tcommand\ngizmo\tcommand\tcommand\n",
            &[SessionMessage::Key(key)],
        );

        let _ = read_frame(&mut reader);
        let (done, apply) = read_done(&mut reader);
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), expected_done);
        assert_eq!(
            apply.strip_suffix('\n').unwrap_or(&apply),
            "APPLY chain=0 execute=0 restore_hex=6769"
        );
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
    fn handle_render_never_emits_common_prefix() {
        // Render metadata is for popup position only; auto-insert travels via
        // FRAME headers in the complete path.  common_prefix must be absent
        // regardless of auto_insert_unambiguous.
        for auto_insert in [true, false] {
            let mut server = test_server();
            server.config.auto_insert_unambiguous = auto_insert;
            let response = server.handle_render(
                RenderParams {
                    prefix: "gi".to_string(),
                    cursor_row: 5,
                    cursor_col: 2,
                    term_cols: 80,
                    term_rows: 24,
                    selected: None,
                },
                b"git-log\tcommand\tcommand\ngit-status\tcommand\tcommand\n",
            );

            match response {
                crate::protocol::Response::Success { metadata, .. } => {
                    let metadata = metadata.unwrap();
                    assert!(
                        !metadata.contains("common_prefix="),
                        "common_prefix must be absent from render metadata \
                         (auto_insert={auto_insert}): {metadata}"
                    );
                }
                other => panic!("unexpected response: {other:?}"),
            }
        }
    }

    #[test]
    fn send_frame_control_char_in_common_prefix_omitted() {
        use crate::app::App;
        use crate::candidate::Candidate;
        let server = test_server();
        let app = App::new(
            vec![Candidate {
                text: "git-log".to_string(),
                description: String::new(),
                kind: "command".to_string(),
            }],
            "g".to_string(),
            5,
            80,
        );
        for ctrl in ["\t", "\r", "\n", "\x1b", "\x7f"] {
            let prefix = format!("git{ctrl}log");
            let mut output = Vec::new();
            server
                .send_frame(&mut output, &app, &[], None, true, Some(&prefix))
                .unwrap();
            let header = String::from_utf8_lossy(&output);
            assert!(
                !header.contains("common_prefix="),
                "control char {ctrl:?} should suppress common_prefix in: {header}"
            );
        }
    }

    #[test]
    fn handle_complete_initial_frame_includes_common_prefix_when_enabled() {
        // With auto_insert_unambiguous=true, the first FRAME header must carry
        // common_prefix= when candidates share a longer prefix than the typed input.
        let mut server = test_server();
        server.config.auto_insert_unambiguous = true;
        let mut reader = run_complete_session(
            &mut server,
            CompleteParams {
                prefix: "gi".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "git-log\tcommand\tcommand\ngit-status\tcommand\tcommand\n",
            &[],
        );
        let mut header = String::new();
        reader.read_line(&mut header).unwrap();
        assert!(
            header.contains("common_prefix=git-"),
            "initial FRAME must contain common_prefix=git- when auto_insert enabled: {header}"
        );
    }

    #[test]
    fn handle_complete_sends_initial_frame_for_new_popup() {
        let mut server = test_server();
        let mut reader = run_complete_session(
            &mut server,
            CompleteParams {
                prefix: "gi".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "git\tcommand\tcommand\ngizmo\tcommand\tcommand\n",
            &[],
        );
        let mut header = String::new();
        reader.read_line(&mut header).unwrap();
        assert!(header.starts_with("FRAME "));
    }

    #[test]
    fn handle_complete_reuse_popup_redraws_popup_without_filter_line() {
        let mut server = test_server();
        let mut reader = run_complete_session(
            &mut server,
            CompleteParams {
                prefix: "gi".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: true,
                shift_tab_sequence: None,
            },
            "git\tcommand\tcommand\ngizmo\tcommand\tcommand\n",
            &[],
        );
        let (header, tty) = read_frame(&mut reader);
        assert!(header.starts_with("FRAME "));
        assert!(tty.contains("┌"));
        assert!(!tty.contains("\u{1b}[6;1Hgi"));
    }

    #[test]
    fn handle_complete_initial_frame_includes_popup_border() {
        let mut server = test_server();
        let mut reader = run_complete_session(
            &mut server,
            CompleteParams {
                prefix: "gi".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "git\tcommand\tcommand\n",
            &[],
        );
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
    }

    #[test]
    fn handle_complete_resize_recomputes_popup_geometry() {
        let mut server = test_server();
        let mut reader = run_complete_session(
            &mut server,
            CompleteParams {
                prefix: "g".to_string(),
                cursor_row: 3,
                cursor_col: 35,
                term_cols: 40,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "git\tcommand\tcommand\ngizmo\tcommand\tcommand\nghub\tcommand\tcommand\nglow\tcommand\tcommand\n",
            &[SessionMessage::Resize {
                cursor_row: 1,
                cursor_col: 20,
                term_cols: 80,
                term_rows: 24,
            }],
        );
        let _ = read_frame(&mut reader);
        let (header, _) = read_frame(&mut reader);
        let frame = TextFrameHeader::parse(header.trim_end()).unwrap();
        assert_eq!(frame.cursor_row, 1);
        assert_eq!(frame.popup_row, 2);
    }

    #[test]
    fn handle_complete_resize_clears_previous_popup_when_column_changes() {
        let mut server = test_server();
        let mut reader = run_complete_session(
            &mut server,
            CompleteParams {
                prefix: "g".to_string(),
                cursor_row: 5,
                cursor_col: 35,
                term_cols: 40,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "git\tcommand\tcommand\nglow\tcommand\tcommand\n",
            &[SessionMessage::Resize {
                cursor_row: 5,
                cursor_col: 20,
                term_cols: 60,
                term_rows: 24,
            }],
        );

        let (first_header, _) = read_frame_bytes(&mut reader);
        let first_frame = TextFrameHeader::parse(first_header.trim_end()).unwrap();
        let expected_clear =
            ui::render::clear_rect_to_bytes(first_frame.popup_row, first_frame.popup_height, 0)
                .unwrap();

        let (_, second_tty_bytes) = read_frame_bytes(&mut reader);
        assert!(
            second_tty_bytes.starts_with(&expected_clear),
            "resize redraw must clear the old popup before drawing at the new column"
        );
    }

    #[test]
    fn handle_complete_tab_after_typing_selects_top_filtered_candidate() {
        let mut server = test_server();
        let mut reader = run_complete_session(
            &mut server,
            CompleteParams {
                prefix: "".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "ab\tcommand\tcommand\nax\tcommand\tcommand\nb\tcommand\tcommand\n",
            &[
                SessionMessage::Key(b"a"),
                SessionMessage::Key(b"\t"),
                SessionMessage::Key(b"\r"),
            ],
        );
        let _ = read_frame(&mut reader);
        let _ = read_frame(&mut reader);
        let _ = read_frame(&mut reader);
        let (done, apply) = read_done(&mut reader);
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), "DONE 0 ab ");
        assert_eq!(
            apply.strip_suffix('\n').unwrap_or(&apply),
            "APPLY chain=1 execute=1 restore_hex="
        );
    }

    #[test]
    fn handle_complete_confirm_after_typing_returns_filter_text() {
        let mut server = test_server();
        let mut reader = run_complete_session(
            &mut server,
            CompleteParams {
                prefix: "".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "ab\tcommand\tcommand\nax\tcommand\tcommand\nb\tcommand\tcommand\n",
            &[SessionMessage::Key(b"a"), SessionMessage::Key(b"\r")],
        );
        let _ = read_frame(&mut reader);
        let _ = read_frame(&mut reader);
        let (done, apply) = read_done(&mut reader);
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), "DONE 1 a");
        assert_eq!(
            apply.strip_suffix('\n').unwrap_or(&apply),
            "APPLY chain=0 execute=0 restore_hex="
        );
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
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
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
        let mut server = test_server();
        let mut reader = run_complete_session(
            &mut server,
            CompleteParams {
                prefix: "".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "ab\tcommand\tcommand\nax\tcommand\tcommand\nb\tcommand\tcommand\n",
            &[
                SessionMessage::Key(b"a"),
                SessionMessage::Key(b"\t"),
                SessionMessage::Key(b" "),
            ],
        );
        let _ = read_frame(&mut reader);
        let _ = read_frame(&mut reader);
        let _ = read_frame(&mut reader);
        let (done, apply) = read_done(&mut reader);
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), "DONE 2 ab ");
        assert_eq!(
            apply.strip_suffix('\n').unwrap_or(&apply),
            "APPLY chain=1 execute=0 restore_hex="
        );
    }

    #[test]
    fn handle_complete_space_after_empty_kind_selection_appends_space() {
        let mut server = test_server();
        let mut reader = run_complete_session(
            &mut server,
            CompleteParams {
                prefix: "gi".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "git\tcommand\t\ngizmo\tcommand\t\n",
            &[SessionMessage::Key(b" ")],
        );
        let _ = read_frame(&mut reader);
        let (done, apply) = read_done(&mut reader);
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), "DONE 2 git ");
        assert_eq!(
            apply.strip_suffix('\n').unwrap_or(&apply),
            "APPLY chain=1 execute=0 restore_hex="
        );
    }

    #[test]
    fn handle_complete_unknown_key_passthroughs_filter_text() {
        assert_passthrough_key("gi", b"\x1b[D", "DONE 3 1b5b44");
    }

    #[test]
    fn handle_complete_ctrl_bindings_passthrough_filter_text() {
        assert_passthrough_key("gi", b"\x01", "DONE 3 01");
        assert_passthrough_key("gi", b"\x05", "DONE 3 05");
        assert_passthrough_key("gi", b"\x0b", "DONE 3 0b");
    }

    #[test]
    fn handle_complete_ctrl_j_passthrough_does_not_inject_newline() {
        let mut server = test_server();
        let mut reader = run_complete_session(
            &mut server,
            CompleteParams {
                prefix: "gi".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "git\tcommand\tcommand\ngizmo\tcommand\tcommand\n",
            &[SessionMessage::Key(b"\n")],
        );
        let _ = read_frame(&mut reader);

        let (done, apply) = read_done(&mut reader);
        assert_eq!(done, "DONE 3 0a\n");
        assert_eq!(apply, "APPLY chain=0 execute=0 restore_hex=6769\n");

        let mut extra = String::new();
        reader.read_line(&mut extra).unwrap();
        assert!(
            extra.is_empty(),
            "unexpected extra protocol line: {extra:?}"
        );
    }

    #[test]
    fn handle_complete_oversized_key_drains_payload_and_passthroughs() {
        assert_passthrough_key(
            "gi",
            b"\x1b[200~git status --short\x1b[201~",
            "DONE 3 1b5b3230307e67697420737461747573202d2d73686f72741b5b3230317e",
        );
    }

    fn assert_immediate_confirm(
        prefix: &str,
        candidates_tsv: &str,
        expected_done: &str,
        expected_apply: &str,
    ) {
        let mut server = test_server();
        let mut reader = run_complete_session(
            &mut server,
            CompleteParams {
                prefix: prefix.to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            candidates_tsv,
            &[SessionMessage::Key(b"\r")],
        );

        let _ = read_frame(&mut reader);
        let (done, apply) = read_done(&mut reader);
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), expected_done);
        assert_eq!(apply.strip_suffix('\n').unwrap_or(&apply), expected_apply);
    }

    #[test]
    fn handle_complete_confirm_with_common_prefix_selects_first() {
        assert_immediate_confirm(
            "fo",
            "foobar\tcommand\tcommand\nfoobaz\tcommand\tcommand\n",
            "DONE 0 foobar ",
            "APPLY chain=1 execute=1 restore_hex=",
        );
    }

    #[test]
    fn handle_complete_confirm_uses_configured_suffix() {
        let mut server = test_server();
        server.config.suffixes = server.config.suffixes.clone().with_override("command", "!");

        let mut input = Vec::new();
        writeln!(&mut input, "KEY 1").unwrap();
        input.extend_from_slice(b"\r");

        let mut reader = BufReader::new(Cursor::new(input));
        let mut writer = Vec::new();
        server.handle_complete(
            &mut reader,
            &mut writer,
            CompleteParams {
                prefix: "ca".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "cargo\tcommand\tcommand\ncargo-add\tcommand\tcommand\n",
        );

        let mut output_reader = BufReader::new(Cursor::new(writer));
        let _ = read_frame(&mut output_reader);
        let (done, apply) = read_done(&mut output_reader);
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), "DONE 0 cargo!");
        assert_eq!(
            apply.strip_suffix('\n').unwrap_or(&apply),
            "APPLY chain=0 execute=1 restore_hex="
        );
    }

    #[test]
    fn handle_complete_accept_single_returns_done_without_frame() {
        let mut server = test_server();
        server.config.suffixes = server.config.suffixes.clone().with_override("command", "!");

        let mut reader = BufReader::new(Cursor::new(Vec::<u8>::new()));
        let mut writer = Vec::new();
        server.handle_complete(
            &mut reader,
            &mut writer,
            CompleteParams {
                prefix: "ca".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: true,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "cargo\tcommand\tcommand\n",
        );

        let output = String::from_utf8(writer).unwrap();
        let mut lines = output.lines();
        assert_eq!(lines.next(), Some("DONE 0 cargo!"));
        assert_eq!(lines.next(), Some("APPLY chain=0 execute=0 restore_hex="));
        assert_eq!(lines.next(), None);
    }

    #[test]
    fn handle_complete_confirm_uses_command_override_for_empty_kind_in_command_position() {
        let mut server = test_server();
        server.config.suffixes = server.config.suffixes.clone().with_override("command", "!");

        let mut input = Vec::new();
        writeln!(&mut input, "KEY 1").unwrap();
        input.extend_from_slice(b"\r");

        let mut reader = BufReader::new(Cursor::new(input));
        let mut writer = Vec::new();
        server.handle_complete(
            &mut reader,
            &mut writer,
            CompleteParams {
                prefix: "gi".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: true,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "git\tcommand\t\ngizmo\tcommand\t\n",
        );

        let mut output_reader = BufReader::new(Cursor::new(writer));
        let _ = read_frame(&mut output_reader);
        let (done, apply) = read_done(&mut output_reader);
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), "DONE 0 git!");
        assert_eq!(
            apply.strip_suffix('\n').unwrap_or(&apply),
            "APPLY chain=0 execute=1 restore_hex="
        );
    }

    #[test]
    fn handle_complete_dismiss_with_space_uses_command_override_for_empty_kind() {
        let mut server = test_server();
        server.config.suffixes = server.config.suffixes.clone().with_override("command", "!");

        let mut input = Vec::new();
        writeln!(&mut input, "KEY 1").unwrap();
        input.extend_from_slice(b" ");

        let mut reader = BufReader::new(Cursor::new(input));
        let mut writer = Vec::new();
        server.handle_complete(
            &mut reader,
            &mut writer,
            CompleteParams {
                prefix: "gi".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: true,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "git\tcommand\t\ngizmo\tcommand\t\n",
        );

        let mut output_reader = BufReader::new(Cursor::new(writer));
        let _ = read_frame(&mut output_reader);
        let (done, apply) = read_done(&mut output_reader);
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), "DONE 2 git! ");
        assert_eq!(
            apply.strip_suffix('\n').unwrap_or(&apply),
            "APPLY chain=1 execute=0 restore_hex="
        );
    }

    #[test]
    fn handle_complete_auto_selects_when_lcp_exceeds_prefix() {
        assert_immediate_confirm(
            "car",
            "cargo\tcommand\tcommand\ncargo-add\tcommand\tcommand\n",
            "DONE 0 cargo ",
            "APPLY chain=1 execute=1 restore_hex=",
        );
    }

    #[test]
    fn handle_complete_cancel_returns_typed_prefix_when_auto_insert_disabled() {
        // With auto_insert_unambiguous=false, cancel should echo back the typed
        // prefix ("gi"), not the extended common prefix ("git-").
        let mut server = test_server();
        server.config.auto_insert_unambiguous = false;
        let mut reader = run_complete_session(
            &mut server,
            CompleteParams {
                prefix: "gi".to_string(),
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                prev_popup_row: None,
                prev_popup_height: None,
                command_position: false,
                accept_single: false,
                reuse_popup: false,
                shift_tab_sequence: None,
            },
            "git-log\tcommand\tcommand\ngit-status\tcommand\tcommand\n",
            &[SessionMessage::Key(b"\x1b")],
        );
        let _ = read_frame(&mut reader);
        let (done, apply) = read_done(&mut reader);
        // filter_text stays at "gi" (= prefix), so Cancel returns empty text.
        // With auto_insert enabled the extended "git-" would have been returned.
        assert_eq!(done.strip_suffix('\n').unwrap_or(&done), "DONE 1 ");
        assert_eq!(
            apply.strip_suffix('\n').unwrap_or(&apply),
            "APPLY chain=0 execute=0 restore_hex="
        );
    }

    // --- Quoted-prefix regression tests (issue #15) ---
    //
    // These tests verify that the Rust daemon correctly handles prefixes that
    // include an opening quote character (IPREFIX in zsh terms).  The shell
    // plugin stores `IPREFIX+PREFIX` as the prefix it sends to the daemon, and
    // candidates are captured with the same IPREFIX prepended by _full_prefix.
    // A prior bug caused prefix_len=0 on the shell side, but these tests pin
    // the protocol contract so a regression would be visible from both ends.

    #[test]
    fn handle_complete_double_quoted_prefix_confirms_candidate() {
        // Simulates `"s<Tab>`: prefix = `"s`, candidate text = `"src/`
        // (the shell plugin captures IPREFIX=`"` into _full_prefix, so the
        // candidate text starts with the opening quote).
        assert_immediate_confirm(
            "\"s",
            "\"src/\t\tdirectory\n",
            "DONE 0 \"src/",
            "APPLY chain=1 execute=1 restore_hex=",
        );
    }

    #[test]
    fn handle_complete_single_quoted_prefix_confirms_candidate() {
        // Simulates `'s<Tab>`: single-quote variant of the same quoting case.
        assert_immediate_confirm(
            "'s",
            "'src/\t\tdirectory\n",
            "DONE 0 'src/",
            "APPLY chain=1 execute=1 restore_hex=",
        );
    }

    #[test]
    fn handle_complete_assignment_prefix_confirms_candidate() {
        // Simulates `FOO=ba<Tab>`: IPREFIX=`FOO=`, PREFIX=`ba`.
        // Ensures IPREFIX-prefixed candidates are returned without duplication.
        assert_immediate_confirm(
            "FOO=ba",
            "FOO=bar\t\t\n",
            "DONE 0 FOO=bar",
            "APPLY chain=0 execute=1 restore_hex=",
        );
    }

    #[test]
    fn handle_complete_option_equals_prefix_confirms_candidate() {
        // Simulates `--opt=va<Tab>`: IPREFIX=`--opt=`, PREFIX=`va`.
        assert_immediate_confirm(
            "--opt=va",
            "--opt=value\t\t\n",
            "DONE 0 --opt=value",
            "APPLY chain=0 execute=1 restore_hex=",
        );
    }

    // --- Candidate cache tests ---

    #[test]
    fn read_prefix_and_candidates_returns_none_for_cache_only_request() {
        let data = Cursor::new(b"gi\nEND\n");
        let mut reader = BufReader::new(data);
        let mut writer = Vec::new();
        let (prefix, tsv) = read_prefix_and_candidates(&mut reader, &mut writer, "render").unwrap();
        assert_eq!(prefix, "gi");
        assert_eq!(tsv, None);
        assert!(writer.is_empty());
    }

    #[test]
    fn read_prefix_and_candidates_returns_some_for_payload_request() {
        let data = Cursor::new(b"gi\ngit\tcommand\tcommand\nEND\n");
        let mut reader = BufReader::new(data);
        let mut writer = Vec::new();
        let (prefix, tsv) = read_prefix_and_candidates(&mut reader, &mut writer, "render").unwrap();
        assert_eq!(prefix, "gi");
        assert_eq!(tsv.as_deref(), Some("git\tcommand\tcommand\n"));
        assert!(writer.is_empty());
    }

    #[test]
    fn read_prefix_and_candidates_rejects_oversized_first_line() {
        let oversized = "x".repeat(1_048_577);
        let data = Cursor::new(format!("gi\n{oversized}\nEND\n"));
        let mut reader = BufReader::new(data);
        let mut writer = Vec::new();
        let result = read_prefix_and_candidates(&mut reader, &mut writer, "render");
        assert!(result.is_err());
        assert_eq!(
            String::from_utf8(writer).unwrap(),
            "ERROR payload too large\n"
        );
    }

    #[test]
    fn read_prefix_and_candidates_rejects_payload_without_end_marker() {
        let data = Cursor::new(b"gi\ngit\tcommand\tcommand\n");
        let mut reader = BufReader::new(data);
        let mut writer = Vec::new();
        let result = read_prefix_and_candidates(&mut reader, &mut writer, "render");
        assert!(result.is_err());
        assert_eq!(String::from_utf8(writer).unwrap(), "ERROR missing END\n");
    }

    #[test]
    fn candidate_cache_get_returns_stored_payload() {
        let mut server = test_server();
        server.store_cached_tsv(
            "123:/tmp:git%20",
            "git".to_string(),
            "git\tcommand\tcommand\n".to_string(),
        );
        assert_eq!(
            server.get_cached_tsv("123:/tmp:git%20", "git"),
            Some("git\tcommand\tcommand\n".to_string())
        );
        assert_eq!(server.get_cached_tsv("missing", "git"), None);
    }

    #[test]
    fn active_popup_get_returns_stored_payload() {
        let mut server = test_server();
        server.store_active_popup(
            "popup-1",
            "git".to_string(),
            "git\tcommand\tcommand\n".to_string(),
        );

        let entry = server.get_active_popup("popup-1").unwrap();

        assert_eq!(entry.prefix, "git");
        assert_eq!(entry.tsv, "git\tcommand\tcommand\n");
    }

    #[test]
    fn handle_text_render_cache_only_prefers_context_key_over_popup_key() {
        let mut server = test_server();
        server.store_cached_tsv(
            "ctx-1",
            "st".to_string(),
            "status\tcommand\tcommand\nstash\tcommand\tcommand\n".to_string(),
        );
        server.store_active_popup(
            "popup-1",
            "gi".to_string(),
            "git\tcommand\tcommand\n".to_string(),
        );

        let output = run_text_request(
            &mut server,
            text_request_input(
                "render 5 2 80 24 context_key=ctx-1 popup_key=popup-1",
                "st",
                None,
            ),
        );

        let (_, tty) = read_text_ok(&output);
        assert!(tty.contains("status"), "tty was: {tty:?}");
        assert!(tty.contains("stash"), "tty was: {tty:?}");
        assert!(!tty.contains("git"), "tty was: {tty:?}");
    }

    #[test]
    fn handle_text_complete_cache_only_prefers_context_key_over_popup_key() {
        let mut server = test_server();
        server.config.suffixes = server.config.suffixes.clone().with_override("command", "!");
        server.store_cached_tsv(
            "ctx-1",
            "st".to_string(),
            "status\tcommand\tcommand\n".to_string(),
        );
        server.store_active_popup(
            "popup-1",
            "gi".to_string(),
            "git\tcommand\tcommand\n".to_string(),
        );

        let request = TextCompleteRequest {
            cursor_row: 5,
            cursor_col: 2,
            term_cols: 80,
            term_rows: 24,
            prev_popup: None,
            command_position: false,
            accept_single: true,
            reuse_token: Some("reuse-1".to_string()),
            shift_tab_sequence: None,
            context_key: Some("ctx-1".to_string()),
            popup_key: Some("popup-1".to_string()),
        };
        let output = run_text_request(
            &mut server,
            text_request_input(&request.header_line(), "st", None),
        );

        let result = read_text_complete_result(&output);
        assert_eq!(result.code, 0);
        assert_eq!(result.text, "status!");
        assert!(!result.chain);
        assert!(!result.execute);
        assert_eq!(result.restore_text, "");
    }

    #[test]
    fn handle_text_complete_popup_cache_accept_single_returns_done_without_frame() {
        let mut server = test_server();
        server.config.suffixes = server.config.suffixes.clone().with_override("command", "!");
        server.store_active_popup(
            "popup-1",
            "ca".to_string(),
            "cargo\tcommand\tcommand\n".to_string(),
        );

        let request = TextCompleteRequest {
            cursor_row: 5,
            cursor_col: 2,
            term_cols: 80,
            term_rows: 24,
            prev_popup: None,
            command_position: false,
            accept_single: true,
            reuse_token: Some("reuse-1".to_string()),
            shift_tab_sequence: None,
            context_key: None,
            popup_key: Some("popup-1".to_string()),
        };
        let output = run_text_request(
            &mut server,
            text_request_input(&request.header_line(), "ca", None),
        );

        let rendered = String::from_utf8_lossy(&output);
        assert!(!rendered.starts_with("FRAME "), "output was: {rendered}");

        let result = read_text_complete_result(&output);
        assert_eq!(result.code, 0);
        assert_eq!(result.text, "cargo!");
        assert!(!result.chain);
        assert!(!result.execute);
        assert_eq!(result.restore_text, "");
    }

    #[test]
    fn candidate_cache_rejects_prefixes_that_do_not_exactly_match_cached_prefix() {
        let mut server = test_server();
        server.store_cached_tsv(
            "123:/tmp:git%20",
            "st".to_string(),
            "status\tcommand\tcommand\nstash\tcommand\tcommand\n".to_string(),
        );

        assert_eq!(
            server.get_cached_tsv("123:/tmp:git%20", "st"),
            Some("status\tcommand\tcommand\nstash\tcommand\tcommand\n".to_string())
        );
        assert_eq!(server.get_cached_tsv("123:/tmp:git%20", "sta"), None);
        assert_eq!(server.get_cached_tsv("123:/tmp:git%20", "s"), None);
        assert_eq!(server.get_cached_tsv("123:/tmp:git%20", "re"), None);
    }

    #[test]
    fn candidate_cache_evicts_oldest_on_overflow() {
        let mut server = test_server();
        let tsv = "git\tcommand\tcommand\n";

        // Fill up to the eviction limit.
        for i in 0..super::CANDIDATE_CACHE_MAX_ENTRIES {
            let key = format!("pid{}:git%20", i);
            server.store_cached_tsv(&key, "".to_string(), tsv.to_string());
        }
        assert_eq!(
            server.candidate_cache.len(),
            super::CANDIDATE_CACHE_MAX_ENTRIES
        );

        // Insert one more entry — the oldest (pid0) should be evicted.
        server.store_cached_tsv("new_key:git%20", "".to_string(), tsv.to_string());
        assert_eq!(
            server.candidate_cache.len(),
            super::CANDIDATE_CACHE_MAX_ENTRIES
        );
        assert!(
            !server.candidate_cache.contains_key("pid0:git%20"),
            "oldest cache entry should have been evicted"
        );
        assert!(
            server.candidate_cache.contains_key("new_key:git%20"),
            "newly inserted entry should be present"
        );
    }

    #[test]
    fn candidate_cache_reinsert_moves_key_to_most_recent_position() {
        let mut server = test_server();
        for i in 0..super::CANDIDATE_CACHE_MAX_ENTRIES {
            server.store_cached_tsv(
                &format!("pid{}:git%20", i),
                format!("git{}", i),
                format!("git{}\tcommand\tcommand\n", i),
            );
        }

        server.store_cached_tsv(
            "pid0:git%20",
            "git0-new".to_string(),
            "git0-new\tcommand\tcommand\n".to_string(),
        );
        server.store_cached_tsv(
            "new_key:git%20",
            "".to_string(),
            "new\tcommand\tcommand\n".to_string(),
        );

        assert!(
            server.candidate_cache.contains_key("pid0:git%20"),
            "reinserted key should not be evicted"
        );
        assert!(
            !server.candidate_cache.contains_key("pid1:git%20"),
            "oldest untouched key should be evicted after pid0 is refreshed"
        );
        assert_eq!(
            server.get_cached_tsv("pid0:git%20", "git0-new"),
            Some("git0-new\tcommand\tcommand\n".to_string())
        );
    }

    #[test]
    fn candidate_cache_hit_moves_key_to_most_recent_position() {
        let mut server = test_server();
        for i in 0..super::CANDIDATE_CACHE_MAX_ENTRIES {
            server.store_cached_tsv(
                &format!("pid{}:git%20", i),
                format!("git{}", i),
                format!("git{}\tcommand\tcommand\n", i),
            );
        }

        assert_eq!(
            server.get_cached_tsv("pid0:git%20", "git0"),
            Some("git0\tcommand\tcommand\n".to_string())
        );
        server.store_cached_tsv(
            "new_key:git%20",
            "".to_string(),
            "new\tcommand\tcommand\n".to_string(),
        );

        assert!(
            server.candidate_cache.contains_key("pid0:git%20"),
            "recently read key should not be evicted"
        );
        assert!(
            !server.candidate_cache.contains_key("pid1:git%20"),
            "oldest untouched key should be evicted after pid0 is read"
        );
    }
}
