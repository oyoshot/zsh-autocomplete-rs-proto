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
        KeyCode::Tab if modifiers.is_empty() => ReadOutcome::Action(bindings.tab),
        KeyCode::Char(' ') if modifiers.is_empty() => ReadOutcome::Action(bindings.space),
        KeyCode::PageDown if modifiers.is_empty() => ReadOutcome::Action(Action::PageDown),
        KeyCode::PageUp if modifiers.is_empty() => ReadOutcome::Action(Action::PageUp),
        KeyCode::Down if modifiers.is_empty() => ReadOutcome::Action(Action::MoveDown),
        KeyCode::Up if modifiers.is_empty() => ReadOutcome::Action(Action::MoveUp),
        KeyCode::Enter if modifiers.is_empty() => ReadOutcome::Action(bindings.enter),
        KeyCode::Esc if modifiers.is_empty() => ReadOutcome::Action(Action::Cancel),
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
            ReadOutcome::Action(Action::Cancel)
        }
        KeyCode::Backspace if modifiers.is_empty() => ReadOutcome::Action(Action::Backspace),
        KeyCode::Char(c) if modifiers.is_empty() || modifiers == KeyModifiers::SHIFT => {
            ReadOutcome::Action(Action::TypeChar(c))
        }
        _ => passthrough_key_event(code, modifiers)
            .map(ReadOutcome::Passthrough)
            .unwrap_or(ReadOutcome::Action(Action::None)),
    }
}

fn passthrough_key_event(code: KeyCode, modifiers: KeyModifiers) -> Option<Vec<u8>> {
    match code {
        KeyCode::Left => Some(special_key_bytes('D', 1, modifiers)),
        KeyCode::Right => Some(special_key_bytes('C', 1, modifiers)),
        KeyCode::Home => Some(special_key_bytes('H', 1, modifiers)),
        KeyCode::End => Some(special_key_bytes('F', 1, modifiers)),
        KeyCode::Delete => Some(tilde_key_bytes(3, modifiers)),
        KeyCode::Insert => Some(tilde_key_bytes(2, modifiers)),
        KeyCode::PageUp => Some(tilde_key_bytes(5, modifiers)),
        KeyCode::PageDown => Some(tilde_key_bytes(6, modifiers)),
        KeyCode::F(n) => function_key_bytes(n, modifiers),
        KeyCode::Enter => Some(prefix_alt(b"\r".to_vec(), modifiers)),
        KeyCode::Backspace => Some(prefix_alt(vec![0x7f], modifiers)),
        KeyCode::Tab => Some(prefix_alt(vec![b'\t'], modifiers)),
        KeyCode::Char(c) => char_key_bytes(c, modifiers),
        _ => None,
    }
}

fn char_key_bytes(c: char, modifiers: KeyModifiers) -> Option<Vec<u8>> {
    let mut bytes = if modifiers.contains(KeyModifiers::CONTROL) {
        vec![control_byte(c)?]
    } else {
        let mut encoded = [0u8; 4];
        c.encode_utf8(&mut encoded).as_bytes().to_vec()
    };

    if modifiers.contains(KeyModifiers::ALT) {
        bytes.insert(0, 0x1b);
    }

    Some(bytes)
}

fn control_byte(c: char) -> Option<u8> {
    match c {
        '@' | ' ' => Some(0x00),
        'a'..='z' => Some((c as u8) - b'a' + 1),
        'A'..='Z' => Some((c as u8) - b'A' + 1),
        '[' => Some(0x1b),
        '\\' => Some(0x1c),
        ']' => Some(0x1d),
        '^' => Some(0x1e),
        '_' => Some(0x1f),
        '?' => Some(0x7f),
        _ => None,
    }
}

fn prefix_alt(mut bytes: Vec<u8>, modifiers: KeyModifiers) -> Vec<u8> {
    if modifiers.contains(KeyModifiers::ALT) {
        bytes.insert(0, 0x1b);
    }
    bytes
}

fn modifier_param(modifiers: KeyModifiers) -> Option<u8> {
    let shift = modifiers.contains(KeyModifiers::SHIFT);
    let alt = modifiers.contains(KeyModifiers::ALT);
    let control = modifiers.contains(KeyModifiers::CONTROL);

    match (shift, alt, control) {
        (false, false, false) => None,
        (true, false, false) => Some(2),
        (false, true, false) => Some(3),
        (true, true, false) => Some(4),
        (false, false, true) => Some(5),
        (true, false, true) => Some(6),
        (false, true, true) => Some(7),
        (true, true, true) => Some(8),
    }
}

fn special_key_bytes(final_byte: char, base: u8, modifiers: KeyModifiers) -> Vec<u8> {
    match modifier_param(modifiers) {
        Some(param) => format!("\x1b[{base};{param}{final_byte}").into_bytes(),
        None => format!("\x1b[{final_byte}").into_bytes(),
    }
}

fn tilde_key_bytes(base: u8, modifiers: KeyModifiers) -> Vec<u8> {
    match modifier_param(modifiers) {
        Some(param) => format!("\x1b[{base};{param}~").into_bytes(),
        None => format!("\x1b[{base}~").into_bytes(),
    }
}

fn function_key_bytes(key: u8, modifiers: KeyModifiers) -> Option<Vec<u8>> {
    let base = match key {
        1 => ("OP", None),
        2 => ("OQ", None),
        3 => ("OR", None),
        4 => ("OS", None),
        5 => ("15", Some('~')),
        6 => ("17", Some('~')),
        7 => ("18", Some('~')),
        8 => ("19", Some('~')),
        9 => ("20", Some('~')),
        10 => ("21", Some('~')),
        11 => ("23", Some('~')),
        12 => ("24", Some('~')),
        _ => return None,
    };

    let bytes = match (base.1, modifier_param(modifiers)) {
        (None, None) => format!("\x1b{}", base.0),
        (None, Some(param)) => format!("\x1b[1;{param}{}", &base.0[1..]),
        (Some(final_byte), None) => format!("\x1b[{}{final_byte}", base.0),
        (Some(final_byte), Some(param)) => format!("\x1b[{};{param}{final_byte}", base.0),
    };

    Some(bytes.into_bytes())
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

    #[test]
    fn parse_key_event_passthroughs_control_chars() {
        let b = default_bindings();
        assert_eq!(
            parse_key_event(KeyCode::Char('a'), KeyModifiers::CONTROL, &b),
            ReadOutcome::Passthrough(vec![0x01])
        );
        assert_eq!(
            parse_key_event(KeyCode::Char('e'), KeyModifiers::CONTROL, &b),
            ReadOutcome::Passthrough(vec![0x05])
        );
    }

    #[test]
    fn parse_key_event_passthroughs_alt_chars() {
        let b = default_bindings();
        assert_eq!(
            parse_key_event(KeyCode::Char('f'), KeyModifiers::ALT, &b),
            ReadOutcome::Passthrough(b"\x1bf".to_vec())
        );
    }

    #[test]
    fn parse_key_event_passthroughs_function_keys() {
        let b = default_bindings();
        assert_eq!(
            parse_key_event(KeyCode::F(1), KeyModifiers::NONE, &b),
            ReadOutcome::Passthrough(b"\x1bOP".to_vec())
        );
        assert_eq!(
            parse_key_event(KeyCode::F(5), KeyModifiers::NONE, &b),
            ReadOutcome::Passthrough(b"\x1b[15~".to_vec())
        );
    }
}
