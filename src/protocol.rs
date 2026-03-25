use std::io::{self, Read, Write};

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
        /// Pre-select the N-th filtered candidate (for Tab-cycle mode).
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
                let selected = if cursor.is_empty() {
                    // Old client without flags byte
                    None
                } else {
                    let flags = cursor[0];
                    cursor = &cursor[1..];
                    if flags & 0x01 != 0 {
                        Some(read_u16(&mut cursor)?)
                    } else {
                        None
                    }
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
}
