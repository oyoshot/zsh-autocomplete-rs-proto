use termwiz::input::{InputEvent, InputParser, KeyCode, KeyEvent, Modifiers};

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

/// Parse raw key bytes into an `Action`.
///
/// Returns `Action::None` for unrecognized input.  Used by the render
/// path where unrecognized keys are simply ignored.
pub fn parse_raw_bytes(bytes: &[u8], bindings: &KeyBindings) -> Action {
    parse_raw_bytes_with_shift_tab(bytes, bindings, None)
}

pub fn parse_raw_bytes_with_shift_tab(
    bytes: &[u8],
    bindings: &KeyBindings,
    extra_shift_tab_sequence: Option<&[u8]>,
) -> Action {
    parse_tty_bytes_with_shift_tab(bytes, bindings, extra_shift_tab_sequence)
        .unwrap_or(Action::None)
}

/// Parse raw key bytes, returning `None` for unrecognized input.
///
/// `None` signals that the caller should pass the key through to the
/// shell unchanged (the popup-session `DONE 3` / passthrough path).
pub fn parse_tty_bytes_with_shift_tab(
    bytes: &[u8],
    bindings: &KeyBindings,
    extra_shift_tab_sequence: Option<&[u8]>,
) -> Option<Action> {
    if bytes == b"\n" {
        return None;
    }

    if extra_shift_tab_sequence == Some(bytes) {
        return Some(bindings.shift_tab);
    }

    let event = parse_input_event(bytes)?;
    map_input_event_to_action(event, bindings)
}

fn parse_input_event(bytes: &[u8]) -> Option<InputEvent> {
    let mut parser = InputParser::new();
    let mut events = parser.parse_as_vec(bytes, false);
    if events.len() == 1 {
        events.pop()
    } else {
        None
    }
}

fn map_input_event_to_action(event: InputEvent, bindings: &KeyBindings) -> Option<Action> {
    match event {
        InputEvent::Key(key) => map_key_event_to_action(key, bindings),
        // Let paste and other non-key events fall back to zsh unchanged.
        _ => None,
    }
}

fn map_key_event_to_action(key: KeyEvent, bindings: &KeyBindings) -> Option<Action> {
    let modifiers = normalize_modifiers(key.modifiers);

    match (key.key, modifiers) {
        (KeyCode::UpArrow | KeyCode::ApplicationUpArrow, Modifiers::NONE) => Some(Action::MoveUp),
        (KeyCode::DownArrow | KeyCode::ApplicationDownArrow, Modifiers::NONE) => {
            Some(Action::MoveDown)
        }
        (KeyCode::PageUp | KeyCode::KeyPadPageUp, Modifiers::NONE) => Some(Action::PageUp),
        (KeyCode::PageDown | KeyCode::KeyPadPageDown, Modifiers::NONE) => Some(Action::PageDown),
        (KeyCode::Escape, Modifiers::NONE) => Some(Action::Cancel),
        (KeyCode::Backspace, Modifiers::NONE) => Some(Action::Backspace),
        (KeyCode::Tab, Modifiers::NONE) => Some(bindings.tab),
        (KeyCode::Tab, Modifiers::SHIFT) => Some(bindings.shift_tab),
        (KeyCode::Enter, Modifiers::NONE) => Some(bindings.enter),
        (KeyCode::Char('c'), Modifiers::CTRL) => Some(Action::Cancel),
        (KeyCode::Char(' '), Modifiers::NONE) => Some(bindings.space),
        (KeyCode::Char(c), Modifiers::NONE) if !c.is_control() => Some(Action::TypeChar(c)),
        _ => None,
    }
}

fn normalize_modifiers(modifiers: Modifiers) -> Modifiers {
    let mut normalized = Modifiers::NONE;

    if modifiers.intersects(Modifiers::SHIFT | Modifiers::LEFT_SHIFT | Modifiers::RIGHT_SHIFT) {
        normalized |= Modifiers::SHIFT;
    }
    if modifiers.intersects(Modifiers::CTRL | Modifiers::LEFT_CTRL | Modifiers::RIGHT_CTRL) {
        normalized |= Modifiers::CTRL;
    }
    if modifiers.intersects(Modifiers::ALT | Modifiers::LEFT_ALT | Modifiers::RIGHT_ALT) {
        normalized |= Modifiers::ALT;
    }

    normalized
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
    fn parse_tty_bytes_leaves_ctrl_a_for_passthrough() {
        let b = default_bindings();
        assert_eq!(parse_tty_bytes_with_shift_tab(b"\x01", &b, None), None);
    }

    #[test]
    fn parse_tty_bytes_leaves_ctrl_j_for_passthrough() {
        let b = default_bindings();
        assert_eq!(parse_tty_bytes_with_shift_tab(b"\n", &b, None), None);
    }

    #[test]
    fn alt_modified_chars_passthrough() {
        let b = default_bindings();
        let event = InputEvent::Key(KeyEvent {
            key: KeyCode::Char('f'),
            modifiers: Modifiers::ALT,
        });

        assert_eq!(map_input_event_to_action(event, &b), None);
    }

    #[test]
    fn paste_events_passthrough() {
        let b = default_bindings();

        assert_eq!(
            map_input_event_to_action(InputEvent::Paste("git status".to_string()), &b),
            None
        );
    }

    #[test]
    fn home_end_delete_keys_passthrough() {
        let b = default_bindings();

        for key in [KeyCode::Home, KeyCode::End, KeyCode::Delete] {
            let event = InputEvent::Key(KeyEvent {
                key,
                modifiers: Modifiers::NONE,
            });
            assert_eq!(map_input_event_to_action(event, &b), None);
        }
    }

    #[test]
    fn function_keys_passthrough() {
        let b = default_bindings();
        let event = InputEvent::Key(KeyEvent {
            key: KeyCode::Function(5),
            modifiers: Modifiers::NONE,
        });

        assert_eq!(map_input_event_to_action(event, &b), None);
    }
}
