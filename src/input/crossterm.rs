use std::{collections::HashMap, sync::LazyLock, time::Duration};

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent};

use crate::{app::Action, input::InputManager, keyboard::Ch8Key};

#[derive(Debug, Default)]
pub(crate) struct CrosstermInputManager {}

impl InputManager for CrosstermInputManager {
    type Err = EventReadingError;

    fn poll_events() -> std::result::Result<Vec<Action>, Self::Err> {
        let mut actions = Vec::new();
        while event::poll(Duration::ZERO).map_err(EventReadingError)? {
            if let Some(action) = Self::read_event()? {
                actions.push(action);
            }
        }
        Ok(actions)
    }

    fn read_event() -> std::result::Result<Option<Action>, Self::Err> {
        event::read()
            .map(Self::action_from_event)
            .map_err(EventReadingError)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("IO error: {0}")]
pub struct EventReadingError(std::io::Error);

impl CrosstermInputManager {
    fn action_from_event(e: Event) -> Option<Action> {
        match e {
            Event::Key(key_event) => Self::handle_key_event(key_event),
            _ => None,
        }
    }

    fn handle_key_event(event: KeyEvent) -> Option<Action> {
        match Ch8Key::try_from(event.code) {
            Ok(ch8_key) => Some(Action::Chip8KeyPress(ch8_key)),
            Err(InputError::NotAVirtualKey(key_code)) => {
                KEYBINDS.get(&key_code).map(|(action, _)| action.clone())
            }
        }
    }
}

const QUIT: KeyCode = KeyCode::Char('q');
const PAUSE: KeyCode = KeyCode::Char('p');
const STEP: KeyCode = KeyCode::Char('s');
const RESET: KeyCode = KeyCode::Char('r');

pub(crate) static KEYBINDS: LazyLock<HashMap<KeyCode, (Action, &'static str)>> =
    LazyLock::new(|| {
        HashMap::from([
            (QUIT, (Action::Quit, "Exit")),
            (PAUSE, (Action::TogglePause, "Pause/Unpause")),
            (STEP, (Action::Step, "Step to next instruction")),
            (RESET, (Action::Reset, "Reset the game")),
        ])
    });

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
            _ => return Err(InputError::NotAVirtualKey(value)),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("{0} is not bound to the virtual keyboard")]
    NotAVirtualKey(KeyCode),
}
