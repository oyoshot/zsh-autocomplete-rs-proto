use std::io::{self, BufRead, Read, Write};

// Command bytes
const CMD_RENDER: u8 = 0x01;
const CMD_CLEAR: u8 = 0x02;
const CMD_PING: u8 = 0x03;
const CMD_SHUTDOWN: u8 = 0x04;

// Response status bytes
const STATUS_SUCCESS: u8 = 0x00;
const STATUS_EMPTY: u8 = 0x01;
const STATUS_ERROR: u8 = 0xFF;

/// Maximum binary protocol payload size (2MB).
const MAX_PAYLOAD_SIZE: usize = 2 * 1024 * 1024;

#[derive(Debug)]
pub enum Request {
    Render {
        prefix: String,
        cursor_row: u16,
        cursor_col: u16,
        term_cols: u16,
        term_rows: u16,
        candidates_tsv: Vec<u8>,
        /// Pre-select the N-th filtered candidate before rendering.
        selected: Option<u16>,
    },
    Clear {
        popup_row: u16,
        popup_height: u16,
        cursor_row: u16,
    },
    Ping,
    Shutdown,
}

#[derive(Debug)]
pub enum Response {
    Success {
        tty_bytes: Vec<u8>,
        metadata: Option<String>,
    },
    Empty,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextRenderRequest {
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub term_cols: u16,
    pub term_rows: u16,
    pub selected: Option<usize>,
    pub context_key: Option<String>,
    pub popup_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextCompleteRequest {
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub term_cols: u16,
    pub term_rows: u16,
    pub prev_popup: Option<(u16, u16)>,
    pub command_position: bool,
    pub accept_single: bool,
    pub reuse_token: Option<String>,
    pub shift_tab_sequence: Option<Vec<u8>>,
    pub context_key: Option<String>,
    pub popup_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextClearRequest {
    pub popup_row: u16,
    pub popup_height: u16,
    pub cursor_row: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextRequest {
    Render(TextRenderRequest),
    Complete(TextCompleteRequest),
    Clear(TextClearRequest),
    Ping,
    Shutdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextFrameHeader {
    pub popup_row: u16,
    pub popup_height: u16,
    pub cursor_row: u16,
    pub common_prefix: Option<String>,
    pub tty_len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextCompleteResult {
    pub code: u8,
    pub text: String,
    pub chain: bool,
    pub execute: bool,
    pub restore_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextSessionRequest {
    Key {
        byte_count: usize,
    },
    Resize {
        cursor_row: u16,
        cursor_col: u16,
        term_cols: u16,
        term_rows: u16,
    },
}

fn write_u32(buf: &mut Vec<u8>, val: u32) {
    buf.extend_from_slice(&val.to_be_bytes());
}

fn write_u16(buf: &mut Vec<u8>, val: u16) {
    buf.extend_from_slice(&val.to_be_bytes());
}

fn read_u32(stream: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf)?;
    Ok(u32::from_be_bytes(buf))
}

fn read_u16(stream: &mut impl Read) -> io::Result<u16> {
    let mut buf = [0u8; 2];
    stream.read_exact(&mut buf)?;
    Ok(u16::from_be_bytes(buf))
}

fn trim_line_end(line: &str) -> &str {
    line.trim_end_matches(['\r', '\n'])
}

fn parse_u16_token(token: &str) -> Option<u16> {
    token.parse().ok()
}

fn encode_hex_bytes(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

impl Request {
    pub fn serialize(&self) -> Vec<u8> {
        let mut payload = Vec::new();
        match self {
            Request::Render {
                prefix,
                cursor_row,
                cursor_col,
                term_cols,
                term_rows,
                candidates_tsv,
                selected,
            } => {
                payload.push(CMD_RENDER);
                let prefix_bytes = prefix.as_bytes();
                write_u16(&mut payload, prefix_bytes.len() as u16);
                payload.extend_from_slice(prefix_bytes);
                write_u16(&mut payload, *cursor_row);
                write_u16(&mut payload, *cursor_col);
                write_u16(&mut payload, *term_cols);
                write_u16(&mut payload, *term_rows);
                // Flags byte: bit 0 = has_selected
                let flags: u8 = if selected.is_some() { 0x01 } else { 0x00 };
                payload.push(flags);
                if let Some(sel) = selected {
                    write_u16(&mut payload, *sel);
                }
                payload.extend_from_slice(candidates_tsv);
            }
            Request::Clear {
                popup_row,
                popup_height,
                cursor_row,
            } => {
                payload.push(CMD_CLEAR);
                write_u16(&mut payload, *popup_row);
                write_u16(&mut payload, *popup_height);
                write_u16(&mut payload, *cursor_row);
            }
            Request::Ping => {
                payload.push(CMD_PING);
            }
            Request::Shutdown => {
                payload.push(CMD_SHUTDOWN);
            }
        }
        let mut msg = Vec::with_capacity(4 + payload.len());
        write_u32(&mut msg, payload.len() as u32);
        msg.extend_from_slice(&payload);
        msg
    }

    pub fn deserialize(stream: &mut impl Read) -> io::Result<Self> {
        let len = read_u32(stream)? as usize;
        if len > MAX_PAYLOAD_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("payload too large: {} bytes", len),
            ));
        }
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf)?;

        if buf.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "empty request"));
        }

        let cmd = buf[0];
        let mut cursor = &buf[1..];

        match cmd {
            CMD_RENDER => {
                let prefix_len = read_u16(&mut cursor)? as usize;
                if cursor.len() < prefix_len {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "prefix length exceeds payload",
                    ));
                }
                let (prefix_bytes, rest) = cursor.split_at(prefix_len);
                cursor = rest;
                let prefix = String::from_utf8(prefix_bytes.to_vec())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                let cursor_row = read_u16(&mut cursor)?;
                let cursor_col = read_u16(&mut cursor)?;
                let term_cols = read_u16(&mut cursor)?;
                let term_rows = read_u16(&mut cursor)?;
                // Flags byte: bit 0 = has_selected
                if cursor.is_empty() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "missing flags byte",
                    ));
                }
                let flags = cursor[0];
                cursor = &cursor[1..];
                let selected = if flags & 0x01 != 0 {
                    Some(read_u16(&mut cursor)?)
                } else {
                    None
                };
                let candidates_tsv = cursor.to_vec();
                Ok(Request::Render {
                    prefix,
                    cursor_row,
                    cursor_col,
                    term_cols,
                    term_rows,
                    candidates_tsv,
                    selected,
                })
            }
            CMD_CLEAR => {
                let popup_row = read_u16(&mut cursor)?;
                let popup_height = read_u16(&mut cursor)?;
                let cursor_row = read_u16(&mut cursor)?;
                Ok(Request::Clear {
                    popup_row,
                    popup_height,
                    cursor_row,
                })
            }
            CMD_PING => Ok(Request::Ping),
            CMD_SHUTDOWN => Ok(Request::Shutdown),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown command: 0x{:02x}", cmd),
            )),
        }
    }
}

impl Response {
    pub fn serialize(&self) -> Vec<u8> {
        let mut payload = Vec::new();
        match self {
            Response::Success {
                tty_bytes,
                metadata,
            } => {
                payload.push(STATUS_SUCCESS);
                write_u32(&mut payload, tty_bytes.len() as u32);
                payload.extend_from_slice(tty_bytes);
                if let Some(meta) = metadata {
                    payload.extend_from_slice(meta.as_bytes());
                }
            }
            Response::Empty => {
                payload.push(STATUS_EMPTY);
            }
            Response::Error(msg) => {
                payload.push(STATUS_ERROR);
                payload.extend_from_slice(msg.as_bytes());
            }
        }
        let mut msg = Vec::with_capacity(4 + payload.len());
        write_u32(&mut msg, payload.len() as u32);
        msg.extend_from_slice(&payload);
        msg
    }

    pub fn deserialize(stream: &mut impl Read) -> io::Result<Self> {
        let len = read_u32(stream)? as usize;
        if len > MAX_PAYLOAD_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("payload too large: {} bytes", len),
            ));
        }
        let mut buf = vec![0u8; len];
        stream.read_exact(&mut buf)?;

        if buf.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "empty response"));
        }

        let status = buf[0];
        let data = &buf[1..];

        match status {
            STATUS_SUCCESS => {
                let mut cursor = data;
                let tty_len = read_u32(&mut cursor)? as usize;
                if cursor.len() < tty_len {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "tty_bytes length exceeds payload",
                    ));
                }
                let tty_bytes = cursor[..tty_len].to_vec();
                let rest = &cursor[tty_len..];
                let metadata = if rest.is_empty() {
                    None
                } else {
                    Some(
                        String::from_utf8(rest.to_vec())
                            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
                    )
                };
                Ok(Response::Success {
                    tty_bytes,
                    metadata,
                })
            }
            STATUS_EMPTY => Ok(Response::Empty),
            STATUS_ERROR => {
                let msg = String::from_utf8_lossy(data).to_string();
                Ok(Response::Error(msg))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown status: 0x{:02x}", status),
            )),
        }
    }

    pub fn write_to(&self, stream: &mut impl Write) -> io::Result<()> {
        stream.write_all(&self.serialize())?;
        stream.flush()
    }
}

impl TextRequest {
    pub fn parse_header(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.as_slice() {
            [
                "render",
                cursor_row,
                cursor_col,
                term_cols,
                term_rows,
                rest @ ..,
            ] => {
                let mut selected = None;
                let mut context_key = None;
                let mut popup_key = None;
                for token in rest {
                    if let Some(value) = token.strip_prefix("selected=") {
                        selected = value.parse().ok();
                    } else if let Some(value) = token.strip_prefix("context_key=") {
                        context_key = Some(value.to_string());
                    } else if let Some(value) = token.strip_prefix("popup_key=") {
                        popup_key = Some(value.to_string());
                    }
                }
                Some(Self::Render(TextRenderRequest {
                    cursor_row: parse_u16_token(cursor_row)?,
                    cursor_col: parse_u16_token(cursor_col)?,
                    term_cols: parse_u16_token(term_cols)?,
                    term_rows: parse_u16_token(term_rows)?,
                    selected,
                    context_key,
                    popup_key,
                }))
            }
            [
                "complete",
                cursor_row,
                cursor_col,
                term_cols,
                term_rows,
                rest @ ..,
            ] => {
                let mut prev_popup_row = None;
                let mut prev_popup_height = None;
                let mut command_position = false;
                let mut accept_single = false;
                let mut reuse_token = None;
                let mut shift_tab_sequence = None;
                let mut context_key = None;
                let mut popup_key = None;
                for token in rest {
                    if let Some(value) = token.strip_prefix("prev_popup_row=") {
                        prev_popup_row = value.parse().ok();
                    } else if let Some(value) = token.strip_prefix("prev_popup_height=") {
                        prev_popup_height = value.parse().ok();
                    } else if let Some(value) = token.strip_prefix("reuse_token=") {
                        reuse_token = Some(value.to_string());
                    } else if let Some(value) = token.strip_prefix("shift_tab_hex=") {
                        shift_tab_sequence = decode_hex_bytes(value);
                    } else if let Some(value) = token.strip_prefix("context_key=") {
                        context_key = Some(value.to_string());
                    } else if let Some(value) = token.strip_prefix("popup_key=") {
                        popup_key = Some(value.to_string());
                    } else if let Some(value) = token.strip_prefix("command_position=") {
                        command_position = value == "1";
                    } else if let Some(value) = token.strip_prefix("accept_single=") {
                        accept_single = value == "1";
                    }
                }
                Some(Self::Complete(TextCompleteRequest {
                    cursor_row: parse_u16_token(cursor_row)?,
                    cursor_col: parse_u16_token(cursor_col)?,
                    term_cols: parse_u16_token(term_cols)?,
                    term_rows: parse_u16_token(term_rows)?,
                    prev_popup: prev_popup_row.zip(prev_popup_height),
                    command_position,
                    accept_single,
                    reuse_token,
                    shift_tab_sequence,
                    context_key,
                    popup_key,
                }))
            }
            ["clear", popup_row, popup_height, cursor_row] => Some(Self::Clear(TextClearRequest {
                popup_row: parse_u16_token(popup_row)?,
                popup_height: parse_u16_token(popup_height)?,
                cursor_row: parse_u16_token(cursor_row)?,
            })),
            ["ping"] => Some(Self::Ping),
            ["shutdown"] => Some(Self::Shutdown),
            _ => None,
        }
    }
}

impl TextSessionRequest {
    pub fn parse(line: &str) -> Option<Self> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts.as_slice() {
            ["KEY", byte_count] => Some(Self::Key {
                byte_count: byte_count.parse().ok()?,
            }),
            ["RESIZE", cursor_row, cursor_col, term_cols, term_rows] => Some(Self::Resize {
                cursor_row: parse_u16_token(cursor_row)?,
                cursor_col: parse_u16_token(cursor_col)?,
                term_cols: parse_u16_token(term_cols)?,
                term_rows: parse_u16_token(term_rows)?,
            }),
            _ => None,
        }
    }

    pub fn header_line(&self) -> String {
        match self {
            Self::Key { byte_count } => format!("KEY {byte_count}"),
            Self::Resize {
                cursor_row,
                cursor_col,
                term_cols,
                term_rows,
            } => format!("RESIZE {cursor_row} {cursor_col} {term_cols} {term_rows}"),
        }
    }
}

impl TextCompleteRequest {
    pub fn header_line(&self) -> String {
        let mut line = format!(
            "complete {} {} {} {}",
            self.cursor_row, self.cursor_col, self.term_cols, self.term_rows
        );
        if let Some((row, height)) = self.prev_popup {
            line.push_str(&format!(" prev_popup_row={row} prev_popup_height={height}"));
        }
        if self.command_position {
            line.push_str(" command_position=1");
        }
        if self.accept_single {
            line.push_str(" accept_single=1");
        }
        if let Some(token) = &self.reuse_token {
            line.push_str(&format!(" reuse_token={token}"));
        }
        if let Some(key) = &self.context_key {
            line.push_str(&format!(" context_key={key}"));
        }
        if let Some(key) = &self.popup_key {
            line.push_str(&format!(" popup_key={key}"));
        }
        if let Some(shift_tab_sequence) = self.shift_tab_sequence.as_deref() {
            line.push_str(&format!(
                " shift_tab_hex={}",
                encode_hex_bytes(shift_tab_sequence)
            ));
        }
        line
    }

    pub fn reuse_popup(&self) -> bool {
        self.reuse_token.is_some()
    }
}

impl TextFrameHeader {
    pub fn parse(header: &str) -> Option<Self> {
        let mut tokens = header.split_whitespace();
        if tokens.next()? != "FRAME" {
            return None;
        }

        let mut popup_row = None;
        let mut popup_height = None;
        let mut cursor_row = None;
        let mut common_prefix = None;
        let mut tty_len = None;

        for token in tokens {
            if let Some(value) = token.strip_prefix("popup_row=") {
                popup_row = value.parse().ok();
            } else if let Some(value) = token.strip_prefix("popup_height=") {
                popup_height = value.parse().ok();
            } else if let Some(value) = token.strip_prefix("cursor_row=") {
                cursor_row = value.parse().ok();
            } else if let Some(value) = token.strip_prefix("common_prefix=") {
                common_prefix = Some(value.to_string());
            } else if !token.contains('=') {
                tty_len = token.parse().ok();
            }
        }

        Some(Self {
            popup_row: popup_row?,
            popup_height: popup_height?,
            cursor_row: cursor_row?,
            common_prefix,
            tty_len: tty_len?,
        })
    }

    pub fn write_to(&self, writer: &mut impl Write) -> io::Result<()> {
        write!(
            writer,
            "FRAME popup_row={} popup_height={} cursor_row={}",
            self.popup_row, self.popup_height, self.cursor_row
        )?;
        if let Some(common_prefix) = &self.common_prefix {
            write!(writer, " common_prefix={common_prefix}")?;
        }
        writeln!(writer, " {}", self.tty_len)
    }
}

impl TextCompleteResult {
    pub fn write_to(&self, mut writer: impl Write) -> io::Result<()> {
        writeln!(writer, "DONE {} {}", self.code, self.text)?;
        writeln!(
            writer,
            "APPLY chain={} execute={} restore_hex={}",
            if self.chain { 1 } else { 0 },
            if self.execute { 1 } else { 0 },
            encode_hex_bytes(self.restore_text.as_bytes())
        )?;
        writer.flush()
    }

    pub fn read_from(reader: &mut impl BufRead, done_header: &str) -> io::Result<Self> {
        let mut parts = done_header.splitn(3, ' ');
        if parts.next() != Some("DONE") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid DONE header: {done_header}"),
            ));
        }

        let code = parts
            .next()
            .and_then(|value| value.parse().ok())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing DONE code"))?;
        let text = parts.next().unwrap_or_default().to_string();

        let mut apply = String::new();
        reader.read_line(&mut apply)?;
        let apply = trim_line_end(&apply);

        if !apply.starts_with("APPLY ") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid APPLY line: {apply}"),
            ));
        }

        let apply_fields = apply.strip_prefix("APPLY ").unwrap_or_default();
        let (flag_fields, restore_text) =
            if let Some((prefix, value)) = apply_fields.split_once(" restore_hex=") {
                (prefix, decode_restore_text_hex(value)?)
            } else if let Some((prefix, value)) = apply_fields.split_once(" restore=") {
                (prefix, value.to_string())
            } else {
                (apply_fields, String::new())
            };

        let mut chain = false;
        let mut execute = false;
        for token in flag_fields.split_whitespace() {
            if let Some(value) = token.strip_prefix("chain=") {
                chain = value == "1";
            } else if let Some(value) = token.strip_prefix("execute=") {
                execute = value == "1";
            }
        }

        Ok(Self {
            code,
            text,
            chain,
            execute,
            restore_text,
        })
    }
}

fn decode_restore_text_hex(hex: &str) -> io::Result<String> {
    if hex.is_empty() {
        return Ok(String::new());
    }

    let bytes = decode_hex_bytes(hex)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid restore_hex"))?;
    String::from_utf8(bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid restore_hex utf-8"))
}

pub fn write_text_ok(writer: &mut impl Write, metadata: &str, tty_len: usize) -> io::Result<()> {
    if metadata.is_empty() {
        writeln!(writer, "OK tty_len={tty_len}")
    } else {
        writeln!(writer, "OK {metadata} tty_len={tty_len}")
    }
}

pub fn decode_hex_bytes(hex: &str) -> Option<Vec<u8>> {
    if hex.is_empty() || hex.len() % 2 != 0 {
        return None;
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

pub fn socket_path() -> std::path::PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        std::path::PathBuf::from(dir).join("zacrs.sock")
    } else {
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        std::path::PathBuf::from(format!("/tmp/zacrs-{}.sock", user))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_request_roundtrip() {
        let req = Request::Render {
            prefix: "gi".to_string(),
            cursor_row: 5,
            cursor_col: 2,
            term_cols: 80,
            term_rows: 24,
            candidates_tsv: b"git\tcommand\tcommand\ngrep\tcommand\tcommand\n".to_vec(),
            selected: None,
        };
        let bytes = req.serialize();
        let parsed = Request::deserialize(&mut &bytes[..]).unwrap();
        match parsed {
            Request::Render {
                prefix,
                cursor_row,
                cursor_col,
                term_cols,
                term_rows,
                candidates_tsv,
                selected,
            } => {
                assert_eq!(prefix, "gi");
                assert_eq!(cursor_row, 5);
                assert_eq!(cursor_col, 2);
                assert_eq!(term_cols, 80);
                assert_eq!(term_rows, 24);
                assert!(candidates_tsv.starts_with(b"git\t"));
                assert_eq!(selected, None);
            }
            _ => panic!("expected Render"),
        }
    }

    #[test]
    fn render_request_with_selected_roundtrip() {
        let req = Request::Render {
            prefix: "gi".to_string(),
            cursor_row: 5,
            cursor_col: 2,
            term_cols: 80,
            term_rows: 24,
            candidates_tsv: b"git\tcommand\tcommand\ngrep\tcommand\tcommand\n".to_vec(),
            selected: Some(1),
        };
        let bytes = req.serialize();
        let parsed = Request::deserialize(&mut &bytes[..]).unwrap();
        match parsed {
            Request::Render {
                selected,
                candidates_tsv,
                ..
            } => {
                assert_eq!(selected, Some(1));
                assert!(candidates_tsv.starts_with(b"git\t"));
            }
            _ => panic!("expected Render"),
        }
    }

    #[test]
    fn clear_request_roundtrip() {
        let req = Request::Clear {
            popup_row: 6,
            popup_height: 12,
            cursor_row: 5,
        };
        let bytes = req.serialize();
        let parsed = Request::deserialize(&mut &bytes[..]).unwrap();
        match parsed {
            Request::Clear {
                popup_row,
                popup_height,
                cursor_row,
            } => {
                assert_eq!(popup_row, 6);
                assert_eq!(popup_height, 12);
                assert_eq!(cursor_row, 5);
            }
            _ => panic!("expected Clear"),
        }
    }

    #[test]
    fn ping_roundtrip() {
        let bytes = Request::Ping.serialize();
        let parsed = Request::deserialize(&mut &bytes[..]).unwrap();
        assert!(matches!(parsed, Request::Ping));
    }

    #[test]
    fn shutdown_roundtrip() {
        let bytes = Request::Shutdown.serialize();
        let parsed = Request::deserialize(&mut &bytes[..]).unwrap();
        assert!(matches!(parsed, Request::Shutdown));
    }

    #[test]
    fn success_response_with_metadata() {
        let resp = Response::Success {
            tty_bytes: vec![0x1b, 0x5b, 0x48],
            metadata: Some("popup_row=6 popup_height=12 cursor_row=5".to_string()),
        };
        let bytes = resp.serialize();
        let parsed = Response::deserialize(&mut &bytes[..]).unwrap();
        match parsed {
            Response::Success {
                tty_bytes,
                metadata,
            } => {
                assert_eq!(tty_bytes, vec![0x1b, 0x5b, 0x48]);
                assert_eq!(
                    metadata.unwrap(),
                    "popup_row=6 popup_height=12 cursor_row=5"
                );
            }
            _ => panic!("expected Success"),
        }
    }

    #[test]
    fn success_response_without_metadata() {
        let resp = Response::Success {
            tty_bytes: vec![0x1b, 0x5b, 0x48],
            metadata: None,
        };
        let bytes = resp.serialize();
        let parsed = Response::deserialize(&mut &bytes[..]).unwrap();
        match parsed {
            Response::Success {
                tty_bytes,
                metadata,
            } => {
                assert_eq!(tty_bytes, vec![0x1b, 0x5b, 0x48]);
                assert!(metadata.is_none());
            }
            _ => panic!("expected Success"),
        }
    }

    #[test]
    fn success_response_with_null_in_tty_bytes() {
        let resp = Response::Success {
            tty_bytes: vec![0x1b, 0x00, 0x5b, 0x00, 0x48],
            metadata: Some("popup_row=6".to_string()),
        };
        let bytes = resp.serialize();
        let parsed = Response::deserialize(&mut &bytes[..]).unwrap();
        match parsed {
            Response::Success {
                tty_bytes,
                metadata,
            } => {
                assert_eq!(tty_bytes, vec![0x1b, 0x00, 0x5b, 0x00, 0x48]);
                assert_eq!(metadata.unwrap(), "popup_row=6");
            }
            _ => panic!("expected Success"),
        }
    }

    #[test]
    fn success_response_null_bytes_no_metadata() {
        let resp = Response::Success {
            tty_bytes: vec![0x00, 0x00, 0x00],
            metadata: None,
        };
        let bytes = resp.serialize();
        let parsed = Response::deserialize(&mut &bytes[..]).unwrap();
        match parsed {
            Response::Success {
                tty_bytes,
                metadata,
            } => {
                assert_eq!(tty_bytes, vec![0x00, 0x00, 0x00]);
                assert!(metadata.is_none());
            }
            _ => panic!("expected Success"),
        }
    }

    #[test]
    fn empty_response_roundtrip() {
        let bytes = Response::Empty.serialize();
        let parsed = Response::deserialize(&mut &bytes[..]).unwrap();
        assert!(matches!(parsed, Response::Empty));
    }

    #[test]
    fn error_response_roundtrip() {
        let resp = Response::Error("something went wrong".to_string());
        let bytes = resp.serialize();
        let parsed = Response::deserialize(&mut &bytes[..]).unwrap();
        match parsed {
            Response::Error(msg) => assert_eq!(msg, "something went wrong"),
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn request_deserialize_rejects_oversized_payload() {
        let fake_len = (MAX_PAYLOAD_SIZE as u32 + 1).to_be_bytes();
        let err = Request::deserialize(&mut &fake_len[..]).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("payload too large"));
    }

    #[test]
    fn render_request_rejects_truncated_prefix_payload() {
        let mut bytes = Vec::new();
        write_u32(&mut bytes, 5);
        bytes.push(CMD_RENDER);
        write_u16(&mut bytes, 4);
        bytes.extend_from_slice(b"ab");

        let err = Request::deserialize(&mut &bytes[..]).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("prefix length exceeds payload"));
    }

    #[test]
    fn response_deserialize_rejects_oversized_payload() {
        let fake_len = (MAX_PAYLOAD_SIZE as u32 + 1).to_be_bytes();
        let err = Response::deserialize(&mut &fake_len[..]).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("payload too large"));
    }

    #[test]
    fn decode_hex_bytes_parses_escape_sequences() {
        assert_eq!(decode_hex_bytes("1b5b5a"), Some(b"\x1b[Z".to_vec()));
        assert_eq!(
            decode_hex_bytes("1b5b32373b323b397e"),
            Some(b"\x1b[27;2;9~".to_vec())
        );
        assert_eq!(decode_hex_bytes("1b5"), None);
    }

    #[test]
    fn text_complete_request_header_roundtrip() {
        let request = TextCompleteRequest {
            cursor_row: 5,
            cursor_col: 2,
            term_cols: 80,
            term_rows: 24,
            prev_popup: Some((6, 12)),
            command_position: true,
            accept_single: true,
            reuse_token: Some("123".to_string()),
            shift_tab_sequence: Some(b"\x1b[Z".to_vec()),
            context_key: Some("ctx".to_string()),
            popup_key: Some("popup".to_string()),
        };

        let parsed = TextRequest::parse_header(&request.header_line()).unwrap();
        assert_eq!(parsed, TextRequest::Complete(request));
    }

    #[test]
    fn text_render_request_header_parses_popup_key() {
        let parsed = TextRequest::parse_header(
            "render 5 2 80 24 selected=1 context_key=ctx popup_key=popup",
        )
        .unwrap();
        assert_eq!(
            parsed,
            TextRequest::Render(TextRenderRequest {
                cursor_row: 5,
                cursor_col: 2,
                term_cols: 80,
                term_rows: 24,
                selected: Some(1),
                context_key: Some("ctx".to_string()),
                popup_key: Some("popup".to_string()),
            })
        );
    }

    #[test]
    fn text_session_resize_header_roundtrip() {
        let request = TextSessionRequest::Resize {
            cursor_row: 8,
            cursor_col: 14,
            term_cols: 120,
            term_rows: 40,
        };

        let parsed = TextSessionRequest::parse(&request.header_line()).unwrap();
        assert_eq!(parsed, request);
    }

    #[test]
    fn text_frame_header_roundtrip() {
        let frame = TextFrameHeader {
            popup_row: 6,
            popup_height: 12,
            cursor_row: 5,
            common_prefix: Some("git-".to_string()),
            tty_len: 128,
        };

        let mut buf = Vec::new();
        frame.write_to(&mut buf).unwrap();
        let header = String::from_utf8(buf).unwrap();

        assert_eq!(TextFrameHeader::parse(header.trim_end()), Some(frame));
    }

    #[test]
    fn text_complete_result_roundtrip() {
        let result = TextCompleteResult {
            code: 2,
            text: "cargo ".to_string(),
            chain: true,
            execute: false,
            restore_text: "cargo ".to_string(),
        };

        let mut buf = Vec::new();
        result.write_to(&mut buf).unwrap();
        let payload = String::from_utf8(buf).unwrap();
        let mut lines = payload.lines();
        let done = lines.next().unwrap();
        let apply = format!("{}\n", lines.next().unwrap());
        let mut reader = io::BufReader::new(apply.as_bytes());

        assert_eq!(
            TextCompleteResult::read_from(&mut reader, done).unwrap(),
            result
        );
    }

    #[test]
    fn text_complete_result_write_to_hex_encodes_restore_text() {
        let result = TextCompleteResult {
            code: 1,
            text: String::new(),
            chain: false,
            execute: false,
            restore_text: "--foo=chain=1 execute=0".to_string(),
        };

        let mut buf = Vec::new();
        result.write_to(&mut buf).unwrap();
        let payload = String::from_utf8(buf).unwrap();

        assert!(payload.contains("restore_hex=2d2d666f6f3d636861696e3d3120657865637574653d30"));
        assert!(!payload.contains("restore=--foo=chain=1 execute=0"));
    }

    #[test]
    fn text_complete_result_read_from_legacy_restore_does_not_set_flags_from_payload() {
        let mut reader =
            io::BufReader::new("APPLY execute=0 restore=--foo=chain=1 execute=1\n".as_bytes());

        let result = TextCompleteResult::read_from(&mut reader, "DONE 1 ").unwrap();

        assert!(!result.chain);
        assert!(!result.execute);
        assert_eq!(result.restore_text, "--foo=chain=1 execute=1");
    }
}
