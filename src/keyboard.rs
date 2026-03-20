use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

#[derive(Debug, Clone, Copy)]
pub(crate) enum Key {
    Zero,
    One,
    Two,
    Three,
    Fout,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
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
            KeyCode::Char('0') => Key::Zero,
            KeyCode::Char('1') => Key::One,
            KeyCode::Char('2') => Key::Two,
            KeyCode::Char('3') => Key::Three,
            KeyCode::Char('4') => Key::Fout,
            KeyCode::Char('5') => Key::Five,
            KeyCode::Char('6') => Key::Six,
            KeyCode::Char('7') => Key::Seven,
            KeyCode::Char('8') => Key::Eight,
            KeyCode::Char('9') => Key::Nine,
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

impl From<Key> for u8 {
    fn from(value: Key) -> Self {
        match value {
            Key::Zero => 0x0,
            Key::One => 0x1,
            Key::Two => 0x2,
            Key::Three => 0x3,
            Key::Fout => 0x4,
            Key::Five => 0x5,
            Key::Six => 0x6,
            Key::Seven => 0x7,
            Key::Eight => 0x8,
            Key::Nine => 0x9,
            Key::A => 0xA,
            Key::B => 0xB,
            Key::C => 0xC,
            Key::D => 0xD,
            Key::E => 0xE,
            Key::F => 0xF,
        }
    }
}

impl TryFrom<u8> for Key {
    type Error = KeyboardError;

    fn try_from(value: u8) -> Result<Self, KeyboardError> {
        Ok(match value {
            0x0 => Key::Zero,
            0x1 => Key::One,
            0x2 => Key::Two,
            0x3 => Key::Three,
            0x4 => Key::Fout,
            0x5 => Key::Five,
            0x6 => Key::Six,
            0x7 => Key::Seven,
            0x8 => Key::Eight,
            0x9 => Key::Nine,
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
        self.key_states[Into::<u8>::into(k) as usize] == KeyState::Down
    }

    pub(crate) fn is_up(&self, k: Key) -> bool {
        self.key_states[Into::<u8>::into(k) as usize] == KeyState::Up
    }

    pub(crate) fn handle_key_event(&mut self, key_event: KeyEvent) {
        if let Result::Ok(key) = Key::try_from(key_event.code) {
            self.key_states[Into::<u8>::into(key) as usize] = match key_event.kind {
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
    InvalidKeyValue(u8),
}
