use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

use crate::registers::VRegister;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Key {
    ZERO,
    ONE,
    TWO,
    THREE,
    FOUR,
    FIVE,
    SIX,
    SEVEN,
    EIGHT,
    NINE,
    A,
    B,
    C,
    D,
    E,
    F,
}

impl TryFrom<KeyCode> for Key {
    type Error = KeyboardError;

    fn try_from(value: KeyCode) -> Result<Self, KeyboardError> {
        Ok(match value {
            KeyCode::Char('0') => Key::ZERO,
            KeyCode::Char('1') => Key::ONE,
            KeyCode::Char('2') => Key::TWO,
            KeyCode::Char('3') => Key::THREE,
            KeyCode::Char('4') => Key::FOUR,
            KeyCode::Char('5') => Key::FIVE,
            KeyCode::Char('6') => Key::SIX,
            KeyCode::Char('7') => Key::SEVEN,
            KeyCode::Char('8') => Key::EIGHT,
            KeyCode::Char('9') => Key::NINE,
            KeyCode::Char('a') => Key::A,
            KeyCode::Char('b') => Key::B,
            KeyCode::Char('c') => Key::C,
            KeyCode::Char('d') => Key::D,
            KeyCode::Char('e') => Key::E,
            KeyCode::Char('f') => Key::F,
            _ => return Err(KeyboardError::KeyUnbound(value)),
        })
    }
}

impl From<Key> for VRegister {
    fn from(value: Key) -> Self {
        match value {
            Key::ZERO => 0x0,
            Key::ONE => 0x1,
            Key::TWO => 0x2,
            Key::THREE => 0x3,
            Key::FOUR => 0x4,
            Key::FIVE => 0x5,
            Key::SIX => 0x6,
            Key::SEVEN => 0x7,
            Key::EIGHT => 0x8,
            Key::NINE => 0x9,
            Key::A => 0xA,
            Key::B => 0xB,
            Key::C => 0xC,
            Key::D => 0xD,
            Key::E => 0xE,
            Key::F => 0xF,
        }
    }
}

impl TryFrom<VRegister> for Key {
    type Error = KeyboardError;

    fn try_from(value: VRegister) -> Result<Self, KeyboardError> {
        Ok(match value {
            0x0 => Key::ZERO,
            0x1 => Key::ONE,
            0x2 => Key::TWO,
            0x3 => Key::THREE,
            0x4 => Key::FOUR,
            0x5 => Key::FIVE,
            0x6 => Key::SIX,
            0x7 => Key::SEVEN,
            0x8 => Key::EIGHT,
            0x9 => Key::NINE,
            0xA => Key::A,
            0xB => Key::B,
            0xC => Key::C,
            0xD => Key::D,
            0xE => Key::E,
            0xF => Key::F,
            _ => return Err(KeyboardError::InvalidKeyValue(value)),
        })
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum KeyState {
    #[default]
    Up,
    Down,
}

#[derive(Debug, Default)]
pub(crate) struct Keyboard {
    key_states: [KeyState; 16],
}

impl Keyboard {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn is_down(&self, k: Key) -> bool {
        self.key_states[Into::<VRegister>::into(k) as usize] == KeyState::Down
    }

    pub(crate) fn is_up(&self, k: Key) -> bool {
        self.key_states[Into::<VRegister>::into(k) as usize] == KeyState::Up
    }

    pub(crate) fn handle_key_event(&mut self, key_event: KeyEvent) {
        if let Result::Ok(key) = Key::try_from(key_event.code) {
            self.key_states[Into::<VRegister>::into(key) as usize] = match key_event.kind {
                KeyEventKind::Press => KeyState::Down,
                KeyEventKind::Release => KeyState::Up,
                KeyEventKind::Repeat => panic!("what is even this state ?"),
            };
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum KeyboardError {
    #[error("{0} is not bound to the virtual keyboard")]
    KeyUnbound(KeyCode),

    #[error("{0} is not a valid key value")]
    InvalidKeyValue(VRegister),
}
