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
}
