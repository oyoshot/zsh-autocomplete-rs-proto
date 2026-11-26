use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::time::Duration;

use filedescriptor::{POLLIN, pollfd};
use rustix::termios::{OptionalActions, Termios};
use signal_hook::SigId;
use signal_hook::consts::SIGWINCH;

use crate::protocol::{
    self, Request, Response, TextCompleteRequest, TextCompleteResult, TextFrameHeader,
    TextSessionRequest,
};
use crate::tty;
use crate::ui;

pub struct RenderResponse {
    pub tty_bytes: Vec<u8>,
    pub metadata: Option<String>,
}

pub enum CompleteSessionOutcome {
    Done(TextCompleteResult),
    CacheMiss,
}

pub fn try_daemon_render(
    prefix: &str,
    cursor_row: u16,
    cursor_col: u16,
    selected: Option<usize>,
    command_position: bool,
    command_context: Option<&str>,
    candidates_raw: &[u8],
) -> Result<RenderResponse, DaemonUnavailable> {
    let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((80, 24));

    let request = Request::Render {
        prefix: prefix.to_string(),
        cursor_row,
        cursor_col,
        term_cols,
        term_rows,
        candidates_tsv: candidates_raw.to_vec(),
        selected: selected.and_then(|s| u16::try_from(s).ok()),
        command_position,
        command_context: command_context.map(str::to_string),
    };

    let response = send_request(&request)?;

    match response {
        Response::Success {
            tty_bytes,
            metadata,
        } => Ok(RenderResponse {
            tty_bytes,
            metadata,
        }),
        Response::Empty => Err(DaemonUnavailable::EmptyResult),
        Response::Error(msg) => Err(DaemonUnavailable::DaemonError(msg)),
    }
}

pub fn try_daemon_clear(
    popup_row: u16,
    popup_height: u16,
    cursor_row: u16,
) -> Result<Vec<u8>, DaemonUnavailable> {
    let request = Request::Clear {
        popup_row,
        popup_height,
        cursor_row,
    };

    let response = send_request(&request)?;

    match response {
        Response::Success {
            tty_bytes,
            metadata: _,
        } => Ok(tty_bytes),
        Response::Empty => Ok(Vec::new()),
        Response::Error(msg) => Err(DaemonUnavailable::DaemonError(msg)),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn try_daemon_complete(
    prefix: &str,
    cursor_row: u16,
    cursor_col: u16,
    candidates_tsv: &str,
    command_position: bool,
    command_context: Option<&str>,
    accept_single: bool,
    candidates_present: bool,
    shift_tab_sequence: Option<Vec<u8>>,
    stale_bytes: Vec<u8>,
    prev_popup: Option<(u16, u16)>,
    reuse_token: Option<&str>,
    context_key: Option<&str>,
    popup_key: Option<&str>,
) -> Result<CompleteSessionOutcome, DaemonUnavailable> {
    let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let stream = connect_stream()?;
    stream.set_read_timeout(Some(Duration::from_secs(60))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(60))).ok();
    let mut writer = stream
        .try_clone()
        .map_err(|_| DaemonUnavailable::NotRunning)?;
    let mut reader = BufReader::new(stream);

    let request = TextCompleteRequest {
        cursor_row,
        cursor_col,
        term_cols,
        term_rows,
        prev_popup,
        command_position,
        command_context: command_context.map(str::to_string),
        accept_single,
        candidates_present,
        reuse_token: reuse_token.map(str::to_string),
        shift_tab_sequence,
        context_key: context_key.map(str::to_string),
        popup_key: popup_key.map(str::to_string),
    };
    writeln!(writer, "{}", request.header_line()).map_err(|_| DaemonUnavailable::NotRunning)?;
    writeln!(writer, "{prefix}").map_err(|_| DaemonUnavailable::NotRunning)?;
    if !candidates_tsv.trim().is_empty() {
        writeln!(writer, "{candidates_tsv}").map_err(|_| DaemonUnavailable::NotRunning)?;
    }
    writeln!(writer, "END").map_err(|_| DaemonUnavailable::NotRunning)?;
    writer.flush().map_err(|_| DaemonUnavailable::NotRunning)?;

    let mut header = String::new();
    read_line_retry(&mut reader, &mut header)?;
    let header = trim_line_end(&header).to_string();
    if header == "CACHE_MISS" {
        return Ok(CompleteSessionOutcome::CacheMiss);
    }

    let result = run_text_popup_session(
        &mut reader,
        &mut writer,
        &header,
        stale_bytes,
        prev_popup,
        cursor_row,
        cursor_col,
    )?;
    Ok(CompleteSessionOutcome::Done(result))
}

pub fn run_text_popup_session<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    initial_header: &str,
    stale_bytes: Vec<u8>,
    prev_popup: Option<(u16, u16)>,
    cursor_row: u16,
    cursor_col: u16,
) -> Result<TextCompleteResult, DaemonUnavailable> {
    if initial_header == "NONE" {
        return Err(DaemonUnavailable::EmptyResult);
    }
    if initial_header.starts_with("DONE ") {
        return TextCompleteResult::read_from(reader, initial_header)
            .map_err(|_| DaemonUnavailable::NotRunning);
    }

    let mut tty = tty::open_tty_rw().map_err(|_| DaemonUnavailable::NotRunning)?;
    let mut tty_writer = tty::open_tty_write().map_err(|_| DaemonUnavailable::NotRunning)?;
    let mut sigwinch_pipe = SigwinchPipe::new()?;

    let mut state = SessionState::with_prev_popup(prev_popup, cursor_row, cursor_col);
    if let Some(result) = state.handle_header(reader, &mut tty_writer, initial_header, true)? {
        return Ok(result);
    }

    let mut raw_mode =
        RawModeGuard::new(tty.try_clone().map_err(|_| DaemonUnavailable::NotRunning)?)?;
    if !stale_bytes.is_empty() {
        let mut pos = 0;
        while pos < stale_bytes.len() {
            let key = extract_single_key(&stale_bytes, &mut pos);
            if key.is_empty() {
                break;
            }
            if let Some(result) = state.send_key(reader, writer, &mut tty_writer, &key)? {
                raw_mode.restore();
                state.clear_popup(&mut tty_writer)?;
                return Ok(result);
            }
        }
    }

    loop {
        match read_session_event(
            &mut tty,
            &mut sigwinch_pipe.reader,
            state.cursor_row,
            state.cursor_col,
        )? {
            SessionEvent::Key(key) => {
                if let Some(result) = state.send_key(reader, writer, &mut tty_writer, &key)? {
                    raw_mode.restore();
                    state.clear_popup(&mut tty_writer)?;
                    return Ok(result);
                }
            }
            SessionEvent::Resize {
                cursor_row,
                cursor_col,
                term_cols,
                term_rows,
            } => {
                if let Some(result) = state.send_resize(
                    reader,
                    writer,
                    &mut tty_writer,
                    (cursor_row, cursor_col, term_cols, term_rows),
                )? {
                    raw_mode.restore();
                    state.clear_popup(&mut tty_writer)?;
                    return Ok(result);
                }
            }
        }
    }
}

fn send_request(request: &Request) -> Result<Response, DaemonUnavailable> {
    let stream = connect_stream()?;
    let mut writer = &stream;
    writer
        .write_all(&request.serialize())
        .map_err(|_| DaemonUnavailable::NotRunning)?;

    let mut reader = BufReader::new(&stream);
    Response::deserialize(&mut reader).map_err(|_| DaemonUnavailable::NotRunning)
}

fn connect_stream() -> Result<UnixStream, DaemonUnavailable> {
    let socket_path = protocol::socket_path();
    let stream = UnixStream::connect(&socket_path).map_err(|_| DaemonUnavailable::NotRunning)?;
    stream
        .set_read_timeout(Some(Duration::from_millis(500)))
        .ok();
    stream
        .set_write_timeout(Some(Duration::from_millis(500)))
        .ok();

    Ok(stream)
}

#[derive(Debug)]
pub enum DaemonUnavailable {
    NotRunning,
    EmptyResult,
    DaemonError(String),
}

impl std::fmt::Display for DaemonUnavailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaemonUnavailable::NotRunning => write!(f, "daemon not running"),
            DaemonUnavailable::EmptyResult => write!(f, "no candidates"),
            DaemonUnavailable::DaemonError(msg) => write!(f, "daemon error: {}", msg),
        }
    }
}

#[derive(Default)]
struct SessionState {
    popup_row: u16,
    popup_height: u16,
    cursor_row: u16,
    cursor_col: u16,
}

impl SessionState {
    fn with_prev_popup(prev_popup: Option<(u16, u16)>, cursor_row: u16, cursor_col: u16) -> Self {
        let (popup_row, popup_height) = prev_popup.unwrap_or((0, 0));
        Self {
            popup_row,
            popup_height,
            cursor_row,
            cursor_col,
        }
    }

    fn handle_frame<R: BufRead>(
        &mut self,
        reader: &mut R,
        tty_writer: &mut File,
        header: &str,
    ) -> Result<(), DaemonUnavailable> {
        let frame = TextFrameHeader::parse(header).ok_or(DaemonUnavailable::NotRunning)?;
        self.popup_row = frame.popup_row;
        self.popup_height = frame.popup_height;
        self.cursor_row = frame.cursor_row;
        relay_tty_bytes(reader, tty_writer, frame.tty_len)?;
        Ok(())
    }

    fn handle_header<R: BufRead>(
        &mut self,
        reader: &mut R,
        tty_writer: &mut File,
        header: &str,
        clear_on_done: bool,
    ) -> Result<Option<TextCompleteResult>, DaemonUnavailable> {
        match header {
            value if value.starts_with("FRAME ") => {
                self.handle_frame(reader, tty_writer, value)?;
                Ok(None)
            }
            value if value.starts_with("DONE ") => {
                if clear_on_done {
                    self.clear_popup(tty_writer)?;
                }
                TextCompleteResult::read_from(reader, value)
                    .map(Some)
                    .map_err(|_| DaemonUnavailable::NotRunning)
            }
            "NONE" => Ok(None),
            _ => Err(DaemonUnavailable::NotRunning),
        }
    }

    fn send_key<R: BufRead, W: Write>(
        &mut self,
        reader: &mut R,
        writer: &mut W,
        tty_writer: &mut File,
        key: &[u8],
    ) -> Result<Option<TextCompleteResult>, DaemonUnavailable> {
        writeln!(
            writer,
            "{}",
            TextSessionRequest::Key {
                byte_count: key.len(),
            }
            .header_line()
        )
        .map_err(|_| DaemonUnavailable::NotRunning)?;
        writer
            .write_all(key)
            .and_then(|_| writer.flush())
            .map_err(|_| DaemonUnavailable::NotRunning)?;

        let mut header = String::new();
        read_line_retry(reader, &mut header)?;
        let header = trim_line_end(&header);
        self.handle_header(reader, tty_writer, header, false)
    }

    fn send_resize<R: BufRead, W: Write>(
        &mut self,
        reader: &mut R,
        writer: &mut W,
        tty_writer: &mut File,
        dims: (u16, u16, u16, u16),
    ) -> Result<Option<TextCompleteResult>, DaemonUnavailable> {
        let (cursor_row, cursor_col, term_cols, term_rows) = dims;
        self.cursor_row = cursor_row;
        self.cursor_col = cursor_col;
        let req = TextSessionRequest::Resize {
            cursor_row,
            cursor_col,
            term_cols,
            term_rows,
        };
        writeln!(writer, "{}", req.header_line())
            .and_then(|_| writer.flush())
            .map_err(|_| DaemonUnavailable::NotRunning)?;

        let mut header = String::new();
        read_line_retry(reader, &mut header)?;
        let header = trim_line_end(&header);
        self.handle_header(reader, tty_writer, header, false)
    }

    fn clear_popup(&self, tty_writer: &mut File) -> Result<(), DaemonUnavailable> {
        if self.popup_height == 0 {
            return Ok(());
        }
        let clear_bytes =
            ui::render::clear_rect_to_bytes(self.popup_row, self.popup_height, self.cursor_row)
                .map_err(|_| DaemonUnavailable::NotRunning)?;
        tty_writer
            .write_all(&clear_bytes)
            .and_then(|_| tty_writer.flush())
            .map_err(|_| DaemonUnavailable::NotRunning)
    }
}

fn trim_line_end(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
}

fn read_line_retry<R: BufRead>(
    reader: &mut R,
    buf: &mut String,
) -> Result<usize, DaemonUnavailable> {
    loop {
        match reader.read_line(buf) {
            Ok(n) => return Ok(n),
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(DaemonUnavailable::NotRunning),
        }
    }
}

fn relay_tty_bytes<R: Read>(
    reader: &mut R,
    tty_writer: &mut File,
    tty_len: usize,
) -> Result<(), DaemonUnavailable> {
    if tty_len == 0 {
        return Ok(());
    }
    let mut buf = vec![0u8; tty_len];
    reader
        .read_exact(&mut buf)
        .map_err(|_| DaemonUnavailable::NotRunning)?;
    tty_writer
        .write_all(&buf)
        .and_then(|_| tty_writer.flush())
        .map_err(|_| DaemonUnavailable::NotRunning)
}

const MAX_ESCAPE_SEQUENCE_LEN: usize = 32;
const BRACKETED_PASTE_START: &[u8] = b"\x1b[200~";
const BRACKETED_PASTE_END: &[u8] = b"\x1b[201~";

enum SessionEvent {
    Key(Vec<u8>),
    Resize {
        cursor_row: u16,
        cursor_col: u16,
        term_cols: u16,
        term_rows: u16,
    },
}

/// Self-pipe that becomes readable whenever SIGWINCH arrives.
///
/// signal-hook installs one process-wide handler that writes a byte with
/// `MSG_DONTWAIT`, preserves errno, uses `SA_RESTART`, and chains to the
/// handler that was installed before it.
struct SigwinchPipe {
    reader: UnixStream,
    id: SigId,
}

impl SigwinchPipe {
    fn new() -> Result<Self, DaemonUnavailable> {
        let (reader, writer) = UnixStream::pair().map_err(|_| DaemonUnavailable::NotRunning)?;
        reader
            .set_nonblocking(true)
            .and_then(|_| writer.set_nonblocking(true))
            .map_err(|_| DaemonUnavailable::NotRunning)?;
        let id = signal_hook::low_level::pipe::register(SIGWINCH, writer)
            .map_err(|_| DaemonUnavailable::NotRunning)?;
        Ok(Self { reader, id })
    }
}

impl Drop for SigwinchPipe {
    fn drop(&mut self) {
        signal_hook::low_level::unregister(self.id);
    }
}

fn is_escape_sequence_complete(buf: &[u8]) -> bool {
    if buf.starts_with(BRACKETED_PASTE_START) && !buf.ends_with(BRACKETED_PASTE_END) {
        return false;
    }

    buf.last()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'~')
        || buf.len() >= MAX_ESCAPE_SEQUENCE_LEN
}

fn readable(fd: &impl AsRawFd) -> pollfd {
    pollfd {
        fd: fd.as_raw_fd(),
        events: POLLIN,
        revents: 0,
    }
}

fn is_readable(entry: &pollfd) -> bool {
    entry.revents != 0
}

/// Waits for read readiness. `filedescriptor::poll` falls back to select(2)
/// on macOS, where poll(2) does not support terminal devices.
fn poll_retry(fds: &mut [pollfd], timeout: Option<Duration>) -> Result<usize, DaemonUnavailable> {
    loop {
        match filedescriptor::poll(fds, timeout) {
            Ok(ready) => return Ok(ready),
            Err(filedescriptor::Error::Poll(err) | filedescriptor::Error::Io(err))
                if err.kind() == io::ErrorKind::Interrupted => {}
            Err(_) => return Err(DaemonUnavailable::NotRunning),
        }
    }
}

fn poll_for_read(fd: &impl AsRawFd) -> Result<bool, DaemonUnavailable> {
    let mut fds = [readable(fd)];
    Ok(poll_retry(&mut fds, Some(Duration::from_millis(50)))? != 0)
}

fn read_session_event(
    tty: &mut File,
    resize: &mut UnixStream,
    current_cursor_row: u16,
    current_cursor_col: u16,
) -> Result<SessionEvent, DaemonUnavailable> {
    loop {
        let mut fds = [readable(tty), readable(resize)];
        poll_retry(&mut fds, None)?;

        if is_readable(&fds[1]) {
            drain_resize_pipe(resize)?;
            let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((80, 24));
            return Ok(SessionEvent::Resize {
                cursor_row: current_cursor_row,
                cursor_col: current_cursor_col,
                term_cols,
                term_rows,
            });
        }

        if is_readable(&fds[0]) {
            return read_key(tty).map(SessionEvent::Key);
        }
    }
}

fn drain_resize_pipe(resize: &mut UnixStream) -> Result<(), DaemonUnavailable> {
    let mut buf = [0u8; 64];
    loop {
        match resize.read(&mut buf) {
            Ok(0) => return Ok(()),
            Ok(_) => continue,
            Err(err) if err.kind() == io::ErrorKind::WouldBlock => return Ok(()),
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(DaemonUnavailable::NotRunning),
        }
    }
}

fn read_byte(reader: &mut impl Read, byte: &mut [u8; 1]) -> Result<usize, DaemonUnavailable> {
    loop {
        match reader.read(byte) {
            Ok(n) => return Ok(n),
            Err(err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(_) => return Err(DaemonUnavailable::NotRunning),
        }
    }
}

fn read_key(tty: &mut File) -> Result<Vec<u8>, DaemonUnavailable> {
    let mut byte = [0u8; 1];
    let n = read_byte(tty, &mut byte)?;
    if n == 0 {
        return Err(DaemonUnavailable::NotRunning);
    }

    let first = byte[0];
    let mut buf = vec![first];
    if first == 0x1b {
        loop {
            if !poll_for_read(tty)? {
                break;
            }
            let n2 = read_byte(tty, &mut byte)?;
            if n2 == 0 {
                break;
            }
            buf.push(byte[0]);
            if is_escape_sequence_complete(&buf) {
                break;
            }
        }
    } else if first >= 0xc0 {
        let extra = if first >= 0xf0 {
            3
        } else if first >= 0xe0 {
            2
        } else if first >= 0xc0 {
            1
        } else {
            0
        };
        for _ in 0..extra {
            let n2 = read_byte(tty, &mut byte)?;
            if n2 == 0 {
                break;
            }
            buf.push(byte[0]);
        }
    }

    Ok(buf)
}

fn extract_single_key(buf: &[u8], pos: &mut usize) -> Vec<u8> {
    if *pos >= buf.len() {
        return Vec::new();
    }
    let first = buf[*pos];
    *pos += 1;
    let mut key = vec![first];
    if first == 0x1b {
        while *pos < buf.len() {
            let next = buf[*pos];
            *pos += 1;
            key.push(next);
            if is_escape_sequence_complete(&key) {
                break;
            }
        }
    } else if first >= 0xc0 {
        let extra = if first >= 0xf0 {
            3
        } else if first >= 0xe0 {
            2
        } else {
            1
        };
        for _ in 0..extra {
            if *pos >= buf.len() {
                break;
            }
            key.push(buf[*pos]);
            *pos += 1;
        }
    }
    key
}

fn retry_on_eintr<T>(mut f: impl FnMut() -> rustix::io::Result<T>) -> Result<T, DaemonUnavailable> {
    loop {
        match f() {
            Ok(value) => return Ok(value),
            Err(rustix::io::Errno::INTR) => continue,
            Err(_) => return Err(DaemonUnavailable::NotRunning),
        }
    }
}

struct RawModeGuard {
    tty: File,
    saved: Termios,
    active: bool,
}

impl RawModeGuard {
    fn new(tty: File) -> Result<Self, DaemonUnavailable> {
        let saved = retry_on_eintr(|| rustix::termios::tcgetattr(&tty))?;
        let mut raw = saved.clone();
        raw.make_raw();
        retry_on_eintr(|| rustix::termios::tcsetattr(&tty, OptionalActions::Now, &raw))?;
        Ok(Self {
            tty,
            saved,
            active: true,
        })
    }

    fn restore(&mut self) {
        if self.active {
            let _ = retry_on_eintr(|| {
                rustix::termios::tcsetattr(&self.tty, OptionalActions::Now, &self.saved)
            });
            self.active = false;
        }
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        self.restore();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io;
    use std::os::fd::OwnedFd;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct InterruptingBuf {
        data: Vec<u8>,
        pos: usize,
        interrupted: bool,
    }

    impl InterruptingBuf {
        fn new(data: &[u8]) -> Self {
            Self {
                data: data.to_vec(),
                pos: 0,
                interrupted: false,
            }
        }
    }

    impl Read for InterruptingBuf {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "synthetic EINTR",
                ));
            }
            let remaining = &self.data[self.pos..];
            if remaining.is_empty() {
                return Ok(0);
            }
            let len = remaining.len().min(buf.len());
            buf[..len].copy_from_slice(&remaining[..len]);
            self.pos += len;
            Ok(len)
        }
    }

    impl BufRead for InterruptingBuf {
        fn fill_buf(&mut self) -> io::Result<&[u8]> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    "synthetic EINTR",
                ));
            }
            Ok(&self.data[self.pos..])
        }

        fn consume(&mut self, amt: usize) {
            self.pos = (self.pos + amt).min(self.data.len());
        }
    }

    fn temp_file_path(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("zacrs-{label}-{nanos}.tmp"))
    }

    #[test]
    fn trim_line_end_preserves_trailing_spaces() {
        assert_eq!(trim_line_end("DONE 0 cargo \n"), "DONE 0 cargo ");
        assert_eq!(trim_line_end("DONE 0 cargo \r\n"), "DONE 0 cargo ");
    }

    #[test]
    fn read_line_retry_retries_on_eintr() {
        let mut reader = InterruptingBuf::new(b"FRAME 1 2 3\n");
        let mut line = String::new();

        let read = read_line_retry(&mut reader, &mut line).unwrap();

        assert_eq!(read, line.len());
        assert_eq!(line, "FRAME 1 2 3\n");
    }

    #[test]
    fn read_done_response_preserves_text_and_restore_spaces() {
        let mut reader = BufReader::new("APPLY chain=1 execute=0 restore=cargo \n".as_bytes());
        let result = TextCompleteResult::read_from(&mut reader, "DONE 2 cargo ").unwrap();
        assert_eq!(result.code, 2);
        assert_eq!(result.text, "cargo ");
        assert!(result.chain);
        assert!(!result.execute);
        assert_eq!(result.restore_text, "cargo ");
    }

    #[test]
    fn initial_done_does_not_require_tty() {
        let mut reader = BufReader::new("APPLY chain=1 execute=1 restore_hex=\n".as_bytes());
        let mut writer = Vec::new();

        let result = run_text_popup_session(
            &mut reader,
            &mut writer,
            "DONE 0 cargo ",
            vec![],
            None,
            0,
            0,
        )
        .unwrap();

        assert_eq!(result.code, 0);
        assert_eq!(result.text, "cargo ");
        assert!(result.chain);
        assert!(result.execute);
    }

    #[test]
    fn initial_done_clears_previous_popup() {
        let path = temp_file_path("initial-done-clear");
        let mut tty_writer = File::create(&path).unwrap();
        let mut reader = BufReader::new("APPLY chain=0 execute=0 restore=\n".as_bytes());
        let mut state = SessionState::with_prev_popup(Some((3, 2)), 5, 0);

        let result = state
            .handle_header(&mut reader, &mut tty_writer, "DONE 1 ", true)
            .unwrap()
            .unwrap();

        drop(tty_writer);
        let bytes = fs::read(&path).unwrap();
        fs::remove_file(&path).ok();

        assert_eq!(result.code, 1);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn extract_single_key_keeps_bracketed_paste_together() {
        let buf = b"\x1b[200~git status --short\x1b[201~x";
        let mut pos = 0;

        let key = extract_single_key(buf, &mut pos);

        assert_eq!(key, b"\x1b[200~git status --short\x1b[201~");
        assert_eq!(pos, key.len());
    }

    fn file_pipe() -> (File, File) {
        let (reader, writer) = io::pipe().unwrap();
        (
            File::from(OwnedFd::from(reader)),
            File::from(OwnedFd::from(writer)),
        )
    }

    #[test]
    fn read_key_keeps_bracketed_paste_together() {
        let (mut reader, mut writer) = file_pipe();
        writer
            .write_all(b"\x1b[200~git status --short\x1b[201~")
            .unwrap();
        drop(writer);

        let key = read_key(&mut reader).unwrap();

        assert_eq!(key, b"\x1b[200~git status --short\x1b[201~");
    }

    #[test]
    fn read_session_event_returns_resize_when_resize_fd_is_ready() {
        let (mut tty_reader, tty_writer) = file_pipe();
        let (mut resize_reader, mut resize_writer) = UnixStream::pair().unwrap();
        resize_reader.set_nonblocking(true).unwrap();
        resize_writer.write_all(&[1]).unwrap();

        let event = read_session_event(&mut tty_reader, &mut resize_reader, 7, 11).unwrap();
        match event {
            SessionEvent::Resize {
                cursor_row,
                cursor_col,
                term_cols,
                term_rows,
            } => {
                assert_eq!(cursor_row, 7);
                assert_eq!(cursor_col, 11);
                assert!(term_cols > 0);
                assert!(term_rows > 0);
            }
            SessionEvent::Key(_) => panic!("expected resize event"),
        }

        drop(resize_writer);
        drop(resize_reader);
        drop(tty_writer);
        drop(tty_reader);
    }
}
