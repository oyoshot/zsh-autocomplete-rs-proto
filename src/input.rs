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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadOutcome {
    Action(Action),
    Passthrough(Vec<u8>),
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
        0x0a | 0x0d => bindings.enter,
        b' ' => bindings.space,
        b if b.is_ascii_graphic() => Action::TypeChar(b as char),
        _ => Action::None,
    }
}

pub fn read_action(bindings: &KeyBindings) -> std::io::Result<Action> {
    Ok(match read_action_with_passthrough(bindings)? {
        ReadOutcome::Action(action) => action,
        ReadOutcome::Passthrough(_) => Action::None,
    })
}

pub fn read_action_with_passthrough(bindings: &KeyBindings) -> std::io::Result<ReadOutcome> {
    if !event::poll(Duration::from_millis(100))? {
        return Ok(ReadOutcome::Action(Action::None));
    }

    match event::read()? {
        Event::Key(KeyEvent {
            code, modifiers, ..
        }) => Ok(parse_key_event(code, modifiers, bindings)),
        Event::Resize(cols, rows) => Ok(ReadOutcome::Action(Action::Resize(cols, rows))),
        _ => Ok(ReadOutcome::Action(Action::None)),
    }
}

fn parse_key_event(code: KeyCode, modifiers: KeyModifiers, bindings: &KeyBindings) -> ReadOutcome {
    match code {
        KeyCode::BackTab => ReadOutcome::Action(bindings.shift_tab),
        KeyCode::Tab => ReadOutcome::Action(bindings.tab),
        KeyCode::Char(' ') => ReadOutcome::Action(bindings.space),
        KeyCode::PageDown => ReadOutcome::Action(Action::PageDown),
        KeyCode::PageUp => ReadOutcome::Action(Action::PageUp),
        KeyCode::Down => ReadOutcome::Action(Action::MoveDown),
        KeyCode::Up => ReadOutcome::Action(Action::MoveUp),
        KeyCode::Enter => ReadOutcome::Action(bindings.enter),
        KeyCode::Esc => ReadOutcome::Action(Action::Cancel),
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            ReadOutcome::Action(Action::Cancel)
        }
        KeyCode::Backspace => ReadOutcome::Action(Action::Backspace),
        KeyCode::Left => ReadOutcome::Passthrough(b"\x1b[D".to_vec()),
        KeyCode::Right => ReadOutcome::Passthrough(b"\x1b[C".to_vec()),
        KeyCode::Home => ReadOutcome::Passthrough(b"\x1b[H".to_vec()),
        KeyCode::End => ReadOutcome::Passthrough(b"\x1b[F".to_vec()),
        KeyCode::Delete => ReadOutcome::Passthrough(b"\x1b[3~".to_vec()),
        KeyCode::Insert => ReadOutcome::Passthrough(b"\x1b[2~".to_vec()),
        KeyCode::Char(c) => ReadOutcome::Action(Action::TypeChar(c)),
        _ => ReadOutcome::Action(Action::None),
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
        assert_eq!(parse_raw_bytes(b"\n", &b), Action::Confirm);
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

    #[test]
    fn parse_key_event_passthroughs_cursor_keys() {
        let b = default_bindings();
        assert_eq!(
            parse_key_event(KeyCode::Left, KeyModifiers::NONE, &b),
            ReadOutcome::Passthrough(b"\x1b[D".to_vec())
        );
        assert_eq!(
            parse_key_event(KeyCode::Right, KeyModifiers::NONE, &b),
            ReadOutcome::Passthrough(b"\x1b[C".to_vec())
        );
        assert_eq!(
            parse_key_event(KeyCode::Home, KeyModifiers::NONE, &b),
            ReadOutcome::Passthrough(b"\x1b[H".to_vec())
        );
        assert_eq!(
            parse_key_event(KeyCode::End, KeyModifiers::NONE, &b),
            ReadOutcome::Passthrough(b"\x1b[F".to_vec())
        );
    }
}
