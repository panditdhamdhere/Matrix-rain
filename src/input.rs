use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

pub enum InputAction {
    None,
    Quit,
    TogglePause,
    ToggleOverlay,
    Resized(u16, u16),
}

pub fn poll_input() -> std::io::Result<InputAction> {
    if !event::poll(Duration::from_millis(0))? {
        return Ok(InputAction::None);
    }

    match event::read()? {
        Event::Resize(w, h) => Ok(InputAction::Resized(w, h)),
        Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
            if matches!(key_event.code, KeyCode::Char('q') | KeyCode::Char('Q')) {
                return Ok(InputAction::Quit);
            }
            if matches!(key_event.code, KeyCode::Char(' ')) {
                return Ok(InputAction::TogglePause);
            }
            if matches!(key_event.code, KeyCode::Char('h') | KeyCode::Char('H')) {
                return Ok(InputAction::ToggleOverlay);
            }
            if matches!(key_event.code, KeyCode::Char('c'))
                && key_event.modifiers.contains(KeyModifiers::CONTROL)
            {
                return Ok(InputAction::Quit);
            }
            Ok(InputAction::None)
        }
        _ => Ok(InputAction::None),
    }
}
