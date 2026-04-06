use std::time::Duration;

use ratatui::crossterm::event::{self, KeyCode, KeyEvent};

use crate::{app::Action, keyboard::Ch8Key};

pub(crate) fn poll_action() -> Result<Vec<Action>, std::io::Error> {
    let mut actions = Vec::new();

    while event::poll(Duration::from_secs(0))? {
        if let event::Event::Key(key_event) = event::read()?
            && let Some(action) = handle_key_event(key_event)
        {
            actions.push(action);
        }
    }

    Ok(actions)
}

pub(crate) fn handle_key_event(event: KeyEvent) -> Option<Action> {
    Some(match Ch8Key::try_from(event.code) {
        Ok(ch8_key) => Action::Chip8KeyPress(ch8_key),
        Err(InputError::KeyNotBound(key_code)) => match key_code {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Char('p') => Action::TogglePause,
            _ => None?,
        },
    })
}

impl TryFrom<KeyCode> for Ch8Key {
    type Error = InputError;

    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        Ok(match value {
            KeyCode::Char('0') => Ch8Key::Zero,
            KeyCode::Char('1') => Ch8Key::One,
            KeyCode::Char('2') => Ch8Key::Two,
            KeyCode::Char('3') => Ch8Key::Three,
            KeyCode::Char('4') => Ch8Key::Four,
            KeyCode::Char('5') => Ch8Key::Five,
            KeyCode::Char('6') => Ch8Key::Six,
            KeyCode::Char('7') => Ch8Key::Seven,
            KeyCode::Char('8') => Ch8Key::Eight,
            KeyCode::Char('9') => Ch8Key::Nine,
            KeyCode::Char('a') => Ch8Key::A,
            KeyCode::Char('b') => Ch8Key::B,
            KeyCode::Char('c') => Ch8Key::C,
            KeyCode::Char('d') => Ch8Key::D,
            KeyCode::Char('e') => Ch8Key::E,
            KeyCode::Char('f') => Ch8Key::F,
            _ => return Err(InputError::KeyNotBound(value)),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("{0} is not bound to the virtual keyboard")]
    KeyNotBound(KeyCode),
}
