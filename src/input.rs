use crossterm::terminal;
use std::io::{self, Read};
use std::os::fd::AsRawFd;
use std::time::Duration;

use crate::config::KeyBindings;

const INPUT_POLL_TIMEOUT: Duration = Duration::from_millis(100);
const ESC_SEQUENCE_TIMEOUT: Duration = Duration::from_millis(20);
const MAX_KEY_BYTES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    MoveDown,
    MoveUp,
    PageDown,
    PageUp,
    Resize(u16, u16),
    Confirm,
    DismissWithSpace,
    Cancel,
    TypeChar(char),
    Backspace,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadOutcome {
    Action(Action),
    Passthrough(Vec<u8>),
}

pub struct TtyInputReader {
    last_size: (u16, u16),
    extra_shift_tab_sequence: Option<Vec<u8>>,
}

impl TtyInputReader {
    pub fn new(extra_shift_tab_sequence: Option<Vec<u8>>) -> io::Result<Self> {
        Ok(Self {
            last_size: terminal::size()?,
            extra_shift_tab_sequence,
        })
    }

    pub fn read<R: Read + AsRawFd>(
        &mut self,
        reader: &mut R,
        bindings: &KeyBindings,
    ) -> io::Result<ReadOutcome> {
        if poll_reader(reader, INPUT_POLL_TIMEOUT)? {
            let bytes = read_key_bytes(reader)?;
            self.last_size = terminal::size()?;
            return Ok(parse_tty_bytes_with_shift_tab(
                &bytes,
                bindings,
                self.extra_shift_tab_sequence.as_deref(),
            )
                .map(ReadOutcome::Action)
                .unwrap_or(ReadOutcome::Passthrough(bytes)));
        }

        match terminal::size()? {
            size if size != self.last_size => {
                self.last_size = size;
                Ok(ReadOutcome::Action(Action::Resize(size.0, size.1)))
            }
            _ => Ok(ReadOutcome::Action(Action::None)),
        }
    }
}

pub fn parse_raw_bytes(bytes: &[u8], bindings: &KeyBindings) -> Action {
    parse_raw_bytes_with_shift_tab(bytes, bindings, None)
}

pub fn parse_raw_bytes_with_shift_tab(
    bytes: &[u8],
    bindings: &KeyBindings,
    extra_shift_tab_sequence: Option<&[u8]>,
) -> Action {
    if extra_shift_tab_sequence == Some(bytes) {
        return bindings.shift_tab;
    }

    match bytes {
        [0x1b, b'[', b'A'] => Action::MoveUp,
        [0x1b, b'[', b'B'] => Action::MoveDown,
        [0x1b, b'[', b'5', b'~'] => Action::PageUp,
        [0x1b, b'[', b'6', b'~'] => Action::PageDown,
        [0x1b, b'[', b'Z'] => bindings.shift_tab,
        [0x1b] => Action::Cancel,
        [b] => parse_single_byte(*b, bindings),
        _ => Action::None,
    }
}

/// Map a single non-ESC byte to an Action.
fn parse_single_byte(byte: u8, bindings: &KeyBindings) -> Action {
    match byte {
        0x03 => Action::Cancel,
        0x08 | 0x7f => Action::Backspace,
        0x09 => bindings.tab,
        0x0d => bindings.enter,
        b' ' => bindings.space,
        b if b.is_ascii_graphic() => Action::TypeChar(b as char),
        _ => Action::None,
    }
}

pub fn parse_tty_bytes_with_shift_tab(
    bytes: &[u8],
    bindings: &KeyBindings,
    extra_shift_tab_sequence: Option<&[u8]>,
) -> Option<Action> {
    match parse_raw_bytes_with_shift_tab(bytes, bindings, extra_shift_tab_sequence) {
        Action::None => parse_utf8_char(bytes).map(Action::TypeChar),
        action => Some(action),
    }
}

fn parse_utf8_char(bytes: &[u8]) -> Option<char> {
    let mut chars = std::str::from_utf8(bytes).ok()?.chars();
    let ch = chars.next()?;
    if chars.next().is_none() {
        Some(ch)
    } else {
        None
    }
}

fn read_key_bytes<R: Read + AsRawFd>(reader: &mut R) -> io::Result<Vec<u8>> {
    let first = read_single_byte(reader)?.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "expected at least one key byte",
        )
    })?;

    let mut bytes = vec![first];
    if first == 0x1b {
        read_until_quiet(reader, &mut bytes, ESC_SEQUENCE_TIMEOUT)?;
    } else if let Some(expected_len) = utf8_sequence_len(first) {
        while bytes.len() < expected_len {
            if !poll_reader(reader, ESC_SEQUENCE_TIMEOUT)? {
                break;
            }
            let Some(next) = read_single_byte(reader)? else {
                break;
            };
            bytes.push(next);
        }
    }

    Ok(bytes)
}

fn read_until_quiet<R: Read + AsRawFd>(
    reader: &mut R,
    bytes: &mut Vec<u8>,
    timeout: Duration,
) -> io::Result<()> {
    while bytes.len() < MAX_KEY_BYTES && poll_reader(reader, timeout)? {
        let Some(next) = read_single_byte(reader)? else {
            break;
        };
        bytes.push(next);
    }

    Ok(())
}

fn read_single_byte<R: Read>(reader: &mut R) -> io::Result<Option<u8>> {
    let mut next = [0u8; 1];
    match reader.read(&mut next)? {
        0 => Ok(None),
        _ => Ok(Some(next[0])),
    }
}

fn utf8_sequence_len(first: u8) -> Option<usize> {
    match first {
        0x00..=0x7f => None,
        0xc0..=0xdf => Some(2),
        0xe0..=0xef => Some(3),
        0xf0..=0xf7 => Some(4),
        _ => None,
    }
}

fn poll_reader<R: AsRawFd>(reader: &R, timeout: Duration) -> io::Result<bool> {
    let timeout_ms = timeout.as_millis().min(i32::MAX as u128) as i32;
    let mut pollfd = libc::pollfd {
        fd: reader.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };

    let ready = unsafe { libc::poll(&mut pollfd, 1, timeout_ms) };
    if ready < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(ready > 0 && (pollfd.revents & libc::POLLIN) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::KeyBindings;
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    use std::thread;

    fn default_bindings() -> KeyBindings {
        KeyBindings::default()
    }

    #[test]
    fn arrow_keys() {
        let b = default_bindings();
        assert_eq!(parse_raw_bytes(b"\x1b[A", &b), Action::MoveUp);
        assert_eq!(parse_raw_bytes(b"\x1b[B", &b), Action::MoveDown);
    }

    #[test]
    fn page_keys() {
        let b = default_bindings();
        assert_eq!(parse_raw_bytes(b"\x1b[5~", &b), Action::PageUp);
        assert_eq!(parse_raw_bytes(b"\x1b[6~", &b), Action::PageDown);
    }

    #[test]
    fn escape_and_ctrl_c() {
        let b = default_bindings();
        assert_eq!(parse_raw_bytes(b"\x1b", &b), Action::Cancel);
        assert_eq!(parse_raw_bytes(b"\x03", &b), Action::Cancel);
    }

    #[test]
    fn enter_tab_space_backspace() {
        let b = default_bindings();
        assert_eq!(parse_raw_bytes(b"\r", &b), Action::Confirm);
        assert_eq!(parse_raw_bytes(b"\n", &b), Action::None);
        assert_eq!(parse_raw_bytes(b"\t", &b), Action::MoveDown);
        assert_eq!(parse_raw_bytes(b" ", &b), Action::DismissWithSpace);
        assert_eq!(parse_raw_bytes(b"\x7f", &b), Action::Backspace);
        assert_eq!(parse_raw_bytes(b"\x08", &b), Action::Backspace);
    }

    #[test]
    fn shift_tab() {
        let b = default_bindings();
        assert_eq!(parse_raw_bytes(b"\x1b[Z", &b), Action::MoveUp);
    }

    #[test]
    fn extra_shift_tab_sequence() {
        let b = default_bindings();
        assert_eq!(
            parse_raw_bytes_with_shift_tab(b"\x1b[27;2;9~", &b, Some(b"\x1b[27;2;9~")),
            Action::MoveUp
        );
    }

    #[test]
    fn ascii_chars() {
        let b = default_bindings();
        assert_eq!(parse_raw_bytes(b"a", &b), Action::TypeChar('a'));
        assert_eq!(parse_raw_bytes(b"Z", &b), Action::TypeChar('Z'));
        assert_eq!(parse_raw_bytes(b"/", &b), Action::TypeChar('/'));
    }

    #[test]
    fn unknown_returns_none() {
        let b = default_bindings();
        assert_eq!(parse_raw_bytes(b"\x1b[X", &b), Action::None);
        assert_eq!(parse_raw_bytes(b"", &b), Action::None);
    }

    #[test]
    fn custom_bindings() {
        let b = KeyBindings {
            enter: Action::MoveDown,
            tab: Action::Confirm,
            ..KeyBindings::default()
        };
        assert_eq!(parse_raw_bytes(b"\r", &b), Action::MoveDown);
        assert_eq!(parse_raw_bytes(b"\t", &b), Action::Confirm);
    }

    #[test]
    fn parse_tty_bytes_handles_utf8_chars() {
        let b = default_bindings();
        assert_eq!(
            parse_tty_bytes_with_shift_tab("あ".as_bytes(), &b, None),
            Some(Action::TypeChar('あ'))
        );
    }

    #[test]
    fn parse_tty_bytes_supports_extra_shift_tab_sequence() {
        let b = default_bindings();
        assert_eq!(
            parse_tty_bytes_with_shift_tab(b"\x1b[27;2;9~", &b, Some(b"\x1b[27;2;9~")),
            Some(Action::MoveUp)
        );
    }

    #[test]
    fn read_key_bytes_preserves_terminal_escape_sequences() {
        let (mut reader, mut writer) = UnixStream::pair().unwrap();
        writer.write_all(b"\x1bOH").unwrap();

        assert_eq!(read_key_bytes(&mut reader).unwrap(), b"\x1bOH".to_vec());
    }

    #[test]
    fn read_key_bytes_waits_for_split_escape_sequences() {
        let (mut reader, mut writer) = UnixStream::pair().unwrap();
        let sender = thread::spawn(move || {
            writer.write_all(b"\x1b").unwrap();
            thread::sleep(Duration::from_millis(5));
            writer.write_all(b"[A").unwrap();
        });

        assert_eq!(read_key_bytes(&mut reader).unwrap(), b"\x1b[A".to_vec());
        sender.join().unwrap();
    }

    #[test]
    fn read_key_bytes_keeps_standalone_escape() {
        let (mut reader, mut writer) = UnixStream::pair().unwrap();
        writer.write_all(b"\x1b").unwrap();

        assert_eq!(read_key_bytes(&mut reader).unwrap(), b"\x1b".to_vec());
    }
}
