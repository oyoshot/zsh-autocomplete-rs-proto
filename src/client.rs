use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use crate::protocol::{self, Request, Response};
use crate::tty;
use crate::ui;

pub struct RenderResponse {
    pub tty_bytes: Vec<u8>,
    pub metadata: Option<String>,
}

pub struct CompleteSessionResult {
    pub code: u8,
    pub text: String,
    pub chain: bool,
    pub execute: bool,
    pub restore_text: String,
}

pub enum CompleteSessionOutcome {
    Done(CompleteSessionResult),
    CacheMiss,
}

pub fn try_daemon_render(
    prefix: &str,
    cursor_row: u16,
    cursor_col: u16,
    selected: Option<usize>,
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
    shift_tab_sequence: Option<Vec<u8>>,
    stale_bytes: Vec<u8>,
    prev_popup: Option<(u16, u16)>,
    reuse_token: Option<&str>,
    context_key: Option<&str>,
) -> Result<CompleteSessionOutcome, DaemonUnavailable> {
    let (term_cols, term_rows) = crossterm::terminal::size().unwrap_or((80, 24));
    let stream = connect_stream()?;
    stream.set_read_timeout(Some(Duration::from_secs(60))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(60))).ok();
    let mut writer = stream
        .try_clone()
        .map_err(|_| DaemonUnavailable::NotRunning)?;
    let mut reader = BufReader::new(stream);

    let mut req = format!("complete {cursor_row} {cursor_col} {term_cols} {term_rows}");
    if let Some((row, height)) = prev_popup {
        req.push_str(&format!(" prev_popup_row={row} prev_popup_height={height}"));
    }
    if let Some(token) = reuse_token {
        req.push_str(&format!(" reuse_token={token}"));
    }
    if let Some(key) = context_key {
        req.push_str(&format!(" context_key={key}"));
    }
    if let Some(shift_tab_hex) = shift_tab_sequence.as_deref().map(encode_hex_bytes) {
        req.push_str(&format!(" shift_tab_hex={shift_tab_hex}"));
    }
    writeln!(writer, "{req}").map_err(|_| DaemonUnavailable::NotRunning)?;
    writeln!(writer, "{prefix}").map_err(|_| DaemonUnavailable::NotRunning)?;
    if !candidates_tsv.trim().is_empty() {
        writeln!(writer, "{candidates_tsv}").map_err(|_| DaemonUnavailable::NotRunning)?;
    }
    writeln!(writer, "END").map_err(|_| DaemonUnavailable::NotRunning)?;
    writer.flush().map_err(|_| DaemonUnavailable::NotRunning)?;

    let mut header = String::new();
    reader
        .read_line(&mut header)
        .map_err(|_| DaemonUnavailable::NotRunning)?;
    let header = header.trim_end().to_string();
    if header == "CACHE_MISS" {
        return Ok(CompleteSessionOutcome::CacheMiss);
    }

    let tty_fd = tty::open_tty_rw().map_err(|_| DaemonUnavailable::NotRunning)?;
    let mut tty_writer = open_tty_writer(&tty_fd).map_err(|_| DaemonUnavailable::NotRunning)?;

    let mut state = SessionState::default();
    match header.as_str() {
        value if value.starts_with("FRAME ") => {
            state.handle_frame(&mut reader, &mut tty_writer, value)?;
        }
        value if value.starts_with("DONE ") => {
            let result = read_done_response(&mut reader, value)?;
            return Ok(CompleteSessionOutcome::Done(result));
        }
        "NONE" => return Err(DaemonUnavailable::EmptyResult),
        _ => return Err(DaemonUnavailable::NotRunning),
    }

    let mut raw_mode = RawModeGuard::new(tty_fd.as_raw_fd())?;
    if !stale_bytes.is_empty() {
        let mut pos = 0;
        while pos < stale_bytes.len() {
            let key = extract_single_key(&stale_bytes, &mut pos);
            if key.is_empty() {
                break;
            }
            if let Some(result) = state.send_key(&mut reader, &mut writer, &mut tty_writer, &key)? {
                raw_mode.restore();
                state.clear_popup(&mut tty_writer)?;
                return Ok(CompleteSessionOutcome::Done(result));
            }
        }
    }

    loop {
        let key = read_key_from_fd(tty_fd.as_raw_fd())?;
        if let Some(result) = state.send_key(&mut reader, &mut writer, &mut tty_writer, &key)? {
            raw_mode.restore();
            state.clear_popup(&mut tty_writer)?;
            return Ok(CompleteSessionOutcome::Done(result));
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
}

impl SessionState {
    fn handle_frame(
        &mut self,
        reader: &mut BufReader<UnixStream>,
        tty_writer: &mut File,
        header: &str,
    ) -> Result<(), DaemonUnavailable> {
        let tty_len = parse_frame_header(self, header).ok_or(DaemonUnavailable::NotRunning)?;
        relay_tty_bytes(reader, tty_writer, tty_len)?;
        Ok(())
    }

    fn send_key(
        &mut self,
        reader: &mut BufReader<UnixStream>,
        writer: &mut UnixStream,
        tty_writer: &mut File,
        key: &[u8],
    ) -> Result<Option<CompleteSessionResult>, DaemonUnavailable> {
        writeln!(writer, "KEY {}", key.len()).map_err(|_| DaemonUnavailable::NotRunning)?;
        writer
            .write_all(key)
            .and_then(|_| writer.flush())
            .map_err(|_| DaemonUnavailable::NotRunning)?;

        let mut header = String::new();
        reader
            .read_line(&mut header)
            .map_err(|_| DaemonUnavailable::NotRunning)?;
        let header = header.trim_end();
        match header {
            value if value.starts_with("FRAME ") => {
                self.handle_frame(reader, tty_writer, value)?;
                Ok(None)
            }
            value if value.starts_with("DONE ") => read_done_response(reader, value).map(Some),
            "NONE" => Ok(None),
            _ => Err(DaemonUnavailable::NotRunning),
        }
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

fn parse_frame_header(state: &mut SessionState, header: &str) -> Option<usize> {
    let mut tty_len = None;
    for token in header.split_whitespace().skip(1) {
        if let Some(value) = token.strip_prefix("popup_row=") {
            state.popup_row = value.parse().ok()?;
        } else if let Some(value) = token.strip_prefix("popup_height=") {
            state.popup_height = value.parse().ok()?;
        } else if let Some(value) = token.strip_prefix("cursor_row=") {
            state.cursor_row = value.parse().ok()?;
        } else if !token.contains('=') {
            tty_len = token.parse().ok();
        }
    }
    tty_len
}

fn read_done_response(
    reader: &mut BufReader<UnixStream>,
    header: &str,
) -> Result<CompleteSessionResult, DaemonUnavailable> {
    let mut parts = header.splitn(3, ' ');
    let _ = parts.next();
    let code = parts
        .next()
        .and_then(|value| value.parse().ok())
        .ok_or(DaemonUnavailable::NotRunning)?;
    let text = parts.next().unwrap_or_default().to_string();

    let mut apply = String::new();
    reader
        .read_line(&mut apply)
        .map_err(|_| DaemonUnavailable::NotRunning)?;
    let apply = apply.trim_end();
    let mut chain = false;
    let mut execute = false;
    let mut restore_text = String::new();
    for token in apply.split_whitespace().skip(1) {
        if let Some(value) = token.strip_prefix("chain=") {
            chain = value == "1";
        } else if let Some(value) = token.strip_prefix("execute=") {
            execute = value == "1";
        }
    }
    if let Some(value) = apply.split_once(" restore=").map(|(_, value)| value) {
        restore_text = value.to_string();
    }

    Ok(CompleteSessionResult {
        code,
        text,
        chain,
        execute,
        restore_text,
    })
}

fn relay_tty_bytes(
    reader: &mut BufReader<UnixStream>,
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

fn encode_hex_bytes(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

fn open_tty_writer(tty_fd: &OwnedFd) -> io::Result<File> {
    let _ = tty_fd;
    File::options().write(true).open("/dev/tty")
}

fn read_key_from_fd(fd: i32) -> Result<Vec<u8>, DaemonUnavailable> {
    let mut byte = [0u8; 1];
    let n = retry_on_eintr(|| unsafe {
        libc::read(fd, byte.as_mut_ptr() as *mut libc::c_void, 1) as libc::c_int
    })?;
    if n == 0 {
        return Err(DaemonUnavailable::NotRunning);
    }

    let first = byte[0];
    let mut buf = vec![first];
    if first == 0x1b {
        let mut tv = libc::timeval {
            tv_sec: 0,
            tv_usec: 50_000,
        };
        loop {
            let mut readfds: libc::fd_set = unsafe { std::mem::zeroed() };
            unsafe { libc::FD_SET(fd, &mut readfds) };
            let ret = retry_on_eintr(|| unsafe {
                libc::select(
                    fd + 1,
                    &mut readfds,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    &mut tv,
                )
            })?;
            if ret == 0 {
                break;
            }
            let n2 = retry_on_eintr(|| unsafe {
                libc::read(fd, byte.as_mut_ptr() as *mut libc::c_void, 1) as libc::c_int
            })?;
            if n2 == 0 {
                break;
            }
            buf.push(byte[0]);
            if byte[0].is_ascii_alphabetic() || byte[0] == b'~' || buf.len() >= 32 {
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
            let n2 = retry_on_eintr(|| unsafe {
                libc::read(fd, byte.as_mut_ptr() as *mut libc::c_void, 1) as libc::c_int
            })?;
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
        while *pos < buf.len() && key.len() < 32 {
            let b = buf[*pos];
            *pos += 1;
            key.push(b);
            if b.is_ascii_alphabetic() || b == b'~' {
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

fn retry_on_eintr(mut f: impl FnMut() -> libc::c_int) -> Result<libc::c_int, DaemonUnavailable> {
    loop {
        let rc = f();
        if rc >= 0 {
            return Ok(rc);
        }
        let err = io::Error::last_os_error();
        if err.kind() != io::ErrorKind::Interrupted {
            return Err(DaemonUnavailable::NotRunning);
        }
    }
}

struct RawModeGuard {
    fd: i32,
    saved: libc::termios,
    active: bool,
}

impl RawModeGuard {
    fn new(fd: i32) -> Result<Self, DaemonUnavailable> {
        let mut termios = unsafe { std::mem::zeroed::<libc::termios>() };
        retry_on_eintr(|| unsafe { libc::tcgetattr(fd, &mut termios) })?;
        let saved = termios;
        unsafe { libc::cfmakeraw(&mut termios) };
        retry_on_eintr(|| unsafe { libc::tcsetattr(fd, libc::TCSANOW, &termios) })?;
        Ok(Self {
            fd,
            saved,
            active: true,
        })
    }

    fn restore(&mut self) {
        if self.active {
            let _ = retry_on_eintr(|| unsafe {
                libc::tcsetattr(self.fd, libc::TCSAFLUSH, &self.saved as *const _)
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
