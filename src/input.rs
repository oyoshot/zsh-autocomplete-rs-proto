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
