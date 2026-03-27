use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use crate::config::KeyBindings;

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

pub fn parse_raw_bytes(bytes: &[u8], bindings: &KeyBindings) -> Action {
    match bytes {
        [0x1b, b'[', b'A'] => Action::MoveUp,
        [0x1b, b'[', b'B'] => Action::MoveDown,
        [0x1b, b'[', b'5', b'~'] => Action::PageUp,
        [0x1b, b'[', b'6', b'~'] => Action::PageDown,
        [0x1b, b'[', b'Z'] => bindings.shift_tab,
        [0x1b] => Action::Cancel, // bare ESC; KeyAssembler handles this differently
        [b] => parse_single_byte(*b, bindings),
        _ => Action::None,
    }
}

/// Map a single non-ESC byte to an Action (shared by `parse_raw_bytes` single-byte arms
/// and `KeyAssembler`).
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

// ---------------------------------------------------------------------------
// KeyAssembler – stateful ESC-sequence assembler for byte-at-a-time input
// ---------------------------------------------------------------------------

/// Result of feeding one byte into [`KeyAssembler`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedResult {
    /// No complete action yet (buffering an ESC sequence).
    None,
    /// One action produced.
    One(Action),
    /// Two actions produced (e.g. ESC resolved as Cancel + the new byte's action).
    Two(Action, Action),
}

#[derive(Debug)]
enum AssemblerState {
    /// No buffered bytes.
    Ground,
    /// Received 0x1B, waiting for `[` or another byte.
    Esc,
    /// Received ESC `[`, accumulating CSI parameter bytes.
    Csi(Vec<u8>),
}

/// Assembles multi-byte escape sequences from individual bytes.
///
/// Zsh's catch-all keymap routes one ASCII byte per widget invocation.
/// The daemon feeds each byte through this assembler, which buffers ESC
/// sequences and emits complete [`Action`]s.
#[derive(Debug)]
pub struct KeyAssembler {
    state: AssemblerState,
}

impl Default for KeyAssembler {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyAssembler {
    pub fn new() -> Self {
        Self {
            state: AssemblerState::Ground,
        }
    }

    /// Feed one byte and return 0–2 actions.
    pub fn feed(&mut self, byte: u8, bindings: &KeyBindings) -> FeedResult {
        match std::mem::replace(&mut self.state, AssemblerState::Ground) {
            AssemblerState::Ground => {
                if byte == 0x1b {
                    self.state = AssemblerState::Esc;
                    FeedResult::None
                } else {
                    FeedResult::One(parse_single_byte(byte, bindings))
                }
            }
            AssemblerState::Esc => {
                if byte == b'[' {
                    self.state = AssemblerState::Csi(Vec::new());
                    FeedResult::None
                } else {
                    // ESC + non-'[': ESC was a standalone Cancel, byte is fresh input.
                    FeedResult::Two(Action::Cancel, parse_single_byte(byte, bindings))
                }
            }
            AssemblerState::Csi(mut buf) => {
                if byte.is_ascii_digit() || byte == b';' {
                    buf.push(byte);
                    self.state = AssemblerState::Csi(buf);
                    FeedResult::None
                } else if byte.is_ascii_alphabetic() || byte == b'~' {
                    // CSI sequence complete – reconstruct into stack buffer.
                    let mut seq = [0u8; 12];
                    seq[0] = 0x1b;
                    seq[1] = b'[';
                    let plen = buf.len().min(seq.len() - 3);
                    seq[2..2 + plen].copy_from_slice(&buf[..plen]);
                    seq[2 + plen] = byte;
                    FeedResult::One(parse_raw_bytes(&seq[..3 + plen], bindings))
                } else {
                    // Unexpected byte inside CSI – discard partial sequence.
                    FeedResult::One(Action::None)
                }
            }
        }
    }

    /// Flush any buffered state (e.g. on connection close or timeout).
    pub fn flush(&mut self) -> FeedResult {
        match std::mem::replace(&mut self.state, AssemblerState::Ground) {
            AssemblerState::Esc => FeedResult::One(Action::Cancel),
            AssemblerState::Csi(_) => FeedResult::One(Action::None),
            AssemblerState::Ground => FeedResult::None,
        }
    }
}

pub fn read_action(bindings: &KeyBindings) -> std::io::Result<Action> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(Action::None);
    }

    match event::read()? {
        Event::Key(KeyEvent {
            code, modifiers, ..
        }) => Ok(match code {
            KeyCode::BackTab => bindings.shift_tab,
            KeyCode::Tab => bindings.tab,
            KeyCode::Char(' ') => bindings.space,
            KeyCode::PageDown => Action::PageDown,
            KeyCode::PageUp => Action::PageUp,
            KeyCode::Down => Action::MoveDown,
            KeyCode::Up => Action::MoveUp,
            KeyCode::Enter => bindings.enter,
            KeyCode::Esc => Action::Cancel,
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => Action::Cancel,
            KeyCode::Backspace => Action::Backspace,
            KeyCode::Char(c) => Action::TypeChar(c),
            _ => Action::None,
        }),
        Event::Resize(cols, rows) => Ok(Action::Resize(cols, rows)),
        _ => Ok(Action::None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::KeyBindings;

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

    // --- KeyAssembler tests ---

    #[test]
    fn assembler_single_byte_actions() {
        let b = default_bindings();
        let mut asm = KeyAssembler::new();
        assert_eq!(asm.feed(b'\t', &b), FeedResult::One(Action::MoveDown));
        assert_eq!(asm.feed(b'\r', &b), FeedResult::One(Action::Confirm));
        assert_eq!(
            asm.feed(b' ', &b),
            FeedResult::One(Action::DismissWithSpace)
        );
        assert_eq!(asm.feed(0x7f, &b), FeedResult::One(Action::Backspace));
        assert_eq!(asm.feed(0x08, &b), FeedResult::One(Action::Backspace));
        assert_eq!(asm.feed(0x03, &b), FeedResult::One(Action::Cancel));
        assert_eq!(asm.feed(b'a', &b), FeedResult::One(Action::TypeChar('a')));
        assert_eq!(asm.feed(b'Z', &b), FeedResult::One(Action::TypeChar('Z')));
    }

    #[test]
    fn assembler_arrow_up() {
        let b = default_bindings();
        let mut asm = KeyAssembler::new();
        assert_eq!(asm.feed(0x1b, &b), FeedResult::None);
        assert_eq!(asm.feed(b'[', &b), FeedResult::None);
        assert_eq!(asm.feed(b'A', &b), FeedResult::One(Action::MoveUp));
    }

    #[test]
    fn assembler_arrow_down() {
        let b = default_bindings();
        let mut asm = KeyAssembler::new();
        assert_eq!(asm.feed(0x1b, &b), FeedResult::None);
        assert_eq!(asm.feed(b'[', &b), FeedResult::None);
        assert_eq!(asm.feed(b'B', &b), FeedResult::One(Action::MoveDown));
    }

    #[test]
    fn assembler_shift_tab() {
        let b = default_bindings();
        let mut asm = KeyAssembler::new();
        assert_eq!(asm.feed(0x1b, &b), FeedResult::None);
        assert_eq!(asm.feed(b'[', &b), FeedResult::None);
        assert_eq!(asm.feed(b'Z', &b), FeedResult::One(Action::MoveUp));
    }

    #[test]
    fn assembler_page_up() {
        let b = default_bindings();
        let mut asm = KeyAssembler::new();
        assert_eq!(asm.feed(0x1b, &b), FeedResult::None);
        assert_eq!(asm.feed(b'[', &b), FeedResult::None);
        assert_eq!(asm.feed(b'5', &b), FeedResult::None);
        assert_eq!(asm.feed(b'~', &b), FeedResult::One(Action::PageUp));
    }

    #[test]
    fn assembler_page_down() {
        let b = default_bindings();
        let mut asm = KeyAssembler::new();
        assert_eq!(asm.feed(0x1b, &b), FeedResult::None);
        assert_eq!(asm.feed(b'[', &b), FeedResult::None);
        assert_eq!(asm.feed(b'6', &b), FeedResult::None);
        assert_eq!(asm.feed(b'~', &b), FeedResult::One(Action::PageDown));
    }

    #[test]
    fn assembler_esc_then_printable() {
        let b = default_bindings();
        let mut asm = KeyAssembler::new();
        assert_eq!(asm.feed(0x1b, &b), FeedResult::None);
        assert_eq!(
            asm.feed(b'a', &b),
            FeedResult::Two(Action::Cancel, Action::TypeChar('a'))
        );
    }

    #[test]
    fn assembler_esc_flush() {
        let b = default_bindings();
        let mut asm = KeyAssembler::new();
        assert_eq!(asm.feed(0x1b, &b), FeedResult::None);
        assert_eq!(asm.flush(), FeedResult::One(Action::Cancel));
        // After flush, assembler is back to ground state
        assert_eq!(asm.feed(b'a', &b), FeedResult::One(Action::TypeChar('a')));
    }

    #[test]
    fn assembler_esc_then_non_bracket() {
        let b = default_bindings();
        let mut asm = KeyAssembler::new();
        assert_eq!(asm.feed(0x1b, &b), FeedResult::None);
        // ESC + 'O' (e.g. SS3 prefix) → Cancel + TypeChar('O')
        assert_eq!(
            asm.feed(b'O', &b),
            FeedResult::Two(Action::Cancel, Action::TypeChar('O'))
        );
    }

    #[test]
    fn assembler_unknown_csi() {
        let b = default_bindings();
        let mut asm = KeyAssembler::new();
        assert_eq!(asm.feed(0x1b, &b), FeedResult::None);
        assert_eq!(asm.feed(b'[', &b), FeedResult::None);
        // Unknown CSI final byte 'X' → None via parse_raw_bytes
        assert_eq!(asm.feed(b'X', &b), FeedResult::One(Action::None));
    }

    #[test]
    fn assembler_unexpected_byte_in_csi() {
        let b = default_bindings();
        let mut asm = KeyAssembler::new();
        assert_eq!(asm.feed(0x1b, &b), FeedResult::None);
        assert_eq!(asm.feed(b'[', &b), FeedResult::None);
        // Non-digit, non-alpha, non-~ byte (e.g. 0x01) → discard
        assert_eq!(asm.feed(0x01, &b), FeedResult::One(Action::None));
    }

    #[test]
    fn assembler_resets_after_sequence() {
        let b = default_bindings();
        let mut asm = KeyAssembler::new();
        // Complete an arrow-up sequence
        asm.feed(0x1b, &b);
        asm.feed(b'[', &b);
        assert_eq!(asm.feed(b'A', &b), FeedResult::One(Action::MoveUp));
        // Next byte should start fresh
        assert_eq!(asm.feed(b'x', &b), FeedResult::One(Action::TypeChar('x')));
    }

    #[test]
    fn assembler_custom_bindings() {
        let b = KeyBindings {
            tab: Action::Confirm,
            shift_tab: Action::MoveDown,
            enter: Action::MoveUp,
            space: Action::Cancel,
        };
        let mut asm = KeyAssembler::new();
        assert_eq!(asm.feed(b'\t', &b), FeedResult::One(Action::Confirm));
        assert_eq!(asm.feed(b'\r', &b), FeedResult::One(Action::MoveUp));
        assert_eq!(asm.feed(b' ', &b), FeedResult::One(Action::Cancel));
        // Shift-Tab via ESC sequence
        asm.feed(0x1b, &b);
        asm.feed(b'[', &b);
        assert_eq!(asm.feed(b'Z', &b), FeedResult::One(Action::MoveDown));
    }

    #[test]
    fn assembler_csi_flush_discards() {
        let b = default_bindings();
        let mut asm = KeyAssembler::new();
        asm.feed(0x1b, &b);
        asm.feed(b'[', &b);
        // Partial CSI, then flush → discard
        assert_eq!(asm.flush(), FeedResult::One(Action::None));
        // Back to ground
        assert_eq!(asm.flush(), FeedResult::None);
    }

    #[test]
    fn assembler_ground_flush_is_none() {
        let mut asm = KeyAssembler::new();
        assert_eq!(asm.flush(), FeedResult::None);
    }

    #[test]
    fn assembler_back_to_back_sequences() {
        let b = default_bindings();
        let mut asm = KeyAssembler::new();
        // Arrow Up
        asm.feed(0x1b, &b);
        asm.feed(b'[', &b);
        assert_eq!(asm.feed(b'A', &b), FeedResult::One(Action::MoveUp));
        // Immediately Arrow Down
        asm.feed(0x1b, &b);
        asm.feed(b'[', &b);
        assert_eq!(asm.feed(b'B', &b), FeedResult::One(Action::MoveDown));
    }
}
