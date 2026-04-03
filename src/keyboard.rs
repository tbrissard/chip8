use std::{
    collections::HashMap,
    fmt::Display,
    time::{Duration, Instant},
};

use ratatui::{
    crossterm::event::{KeyCode, KeyEventKind},
    layout::{Constraint, Layout},
    style::{Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Widget},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Ch8Key {
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

impl Display for Ch8Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ch8Key::Zero => write!(f, "0"),
            Ch8Key::One => write!(f, "1"),
            Ch8Key::Two => write!(f, "2"),
            Ch8Key::Three => write!(f, "3"),
            Ch8Key::Fout => write!(f, "4"), // Assuming "Fout" is a typo for "Four"
            Ch8Key::Five => write!(f, "5"),
            Ch8Key::Six => write!(f, "6"),
            Ch8Key::Seven => write!(f, "7"),
            Ch8Key::Eight => write!(f, "8"),
            Ch8Key::Nine => write!(f, "9"),
            Ch8Key::A => write!(f, "A"),
            Ch8Key::B => write!(f, "B"),
            Ch8Key::C => write!(f, "C"),
            Ch8Key::D => write!(f, "D"),
            Ch8Key::E => write!(f, "E"),
            Ch8Key::F => write!(f, "F"),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) enum KeyState {
    #[default]
    Up,
    Down(Instant),
}

#[derive(Debug)]
pub(crate) struct Ch8Keyboard {
    states: HashMap<Ch8Key, KeyState>,
}

impl Default for Ch8Keyboard {
    fn default() -> Self {
        let states = HashMap::from([
            (Ch8Key::Zero, KeyState::Up),
            (Ch8Key::One, KeyState::Up),
            (Ch8Key::Two, KeyState::Up),
            (Ch8Key::Three, KeyState::Up),
            (Ch8Key::Fout, KeyState::Up),
            (Ch8Key::Five, KeyState::Up),
            (Ch8Key::Six, KeyState::Up),
            (Ch8Key::Seven, KeyState::Up),
            (Ch8Key::Eight, KeyState::Up),
            (Ch8Key::Nine, KeyState::Up),
            (Ch8Key::A, KeyState::Up),
            (Ch8Key::B, KeyState::Up),
            (Ch8Key::C, KeyState::Up),
            (Ch8Key::D, KeyState::Up),
            (Ch8Key::E, KeyState::Up),
            (Ch8Key::F, KeyState::Up),
        ]);
        Self { states }
    }
}

impl Ch8Keyboard {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn is_down(&self, k: Ch8Key) -> bool {
        matches!(
            *self.states.get(&k).expect("hashmap is initialized"),
            KeyState::Down(_)
        )
    }

    pub(super) fn is_up(&self, k: Ch8Key) -> bool {
        *self.states.get(&k).expect("hashmap is initialized") == KeyState::Up
    }

    pub(super) fn handle_ch8_key_event(&mut self, key: Ch8Key, kind: KeyEventKind) {
        *self.states.get_mut(&key).expect("hashmap is initialized") = match kind {
            KeyEventKind::Press => KeyState::Down(Instant::now()),
            KeyEventKind::Release => KeyState::Up,
            KeyEventKind::Repeat => panic!("should not happen ?"),
        };
    }

    /// Simulate the release of keys that have been pressed
    pub(super) fn release_keys(&mut self) {
        const PRESS_DURATION: Duration = Duration::from_millis(100);

        self.states.values_mut().for_each(|v| {
            if let KeyState::Down(instant) = v
                && instant.elapsed() > PRESS_DURATION
            {
                *v = KeyState::Up
            }
        });
    }
}

impl Widget for &Ch8Keyboard {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let title = Line::from("Keyboard").bold().centered();
        let block = Block::bordered().title(title).border_set(border::THICK);
        let block_area = block.inner(area);

        let layout = Layout::horizontal(vec![Constraint::Length(3); 16])
            .spacing(3)
            .split(block_area);

        let regular = Style::default();
        let pressed = regular.add_modifier(Modifier::BOLD | Modifier::REVERSED);

        for (i, (k, v)) in self.states.iter().enumerate() {
            Span::styled(
                k.to_string(),
                if *v == KeyState::Up { regular } else { pressed },
            )
            .render(layout[i], buf);
        }

        block.render(area, buf);
    }
}

impl TryFrom<KeyCode> for Ch8Key {
    type Error = KeyboardError;

    fn try_from(value: KeyCode) -> Result<Self, KeyboardError> {
        Ok(match value {
            KeyCode::Char('0') => Ch8Key::Zero,
            KeyCode::Char('1') => Ch8Key::One,
            KeyCode::Char('2') => Ch8Key::Two,
            KeyCode::Char('3') => Ch8Key::Three,
            KeyCode::Char('4') => Ch8Key::Fout,
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
            _ => return Err(KeyboardError::KeyNotBound(value)),
        })
    }
}

impl From<Ch8Key> for u8 {
    fn from(value: Ch8Key) -> Self {
        match value {
            Ch8Key::Zero => 0x0,
            Ch8Key::One => 0x1,
            Ch8Key::Two => 0x2,
            Ch8Key::Three => 0x3,
            Ch8Key::Fout => 0x4,
            Ch8Key::Five => 0x5,
            Ch8Key::Six => 0x6,
            Ch8Key::Seven => 0x7,
            Ch8Key::Eight => 0x8,
            Ch8Key::Nine => 0x9,
            Ch8Key::A => 0xA,
            Ch8Key::B => 0xB,
            Ch8Key::C => 0xC,
            Ch8Key::D => 0xD,
            Ch8Key::E => 0xE,
            Ch8Key::F => 0xF,
        }
    }
}

impl TryFrom<u8> for Ch8Key {
    type Error = KeyboardError;

    fn try_from(value: u8) -> Result<Self, KeyboardError> {
        Ok(match value {
            0x0 => Ch8Key::Zero,
            0x1 => Ch8Key::One,
            0x2 => Ch8Key::Two,
            0x3 => Ch8Key::Three,
            0x4 => Ch8Key::Fout,
            0x5 => Ch8Key::Five,
            0x6 => Ch8Key::Six,
            0x7 => Ch8Key::Seven,
            0x8 => Ch8Key::Eight,
            0x9 => Ch8Key::Nine,
            0xA => Ch8Key::A,
            0xB => Ch8Key::B,
            0xC => Ch8Key::C,
            0xD => Ch8Key::D,
            0xE => Ch8Key::E,
            0xF => Ch8Key::F,
            _ => return Err(KeyboardError::InvalidKeyValue(value)),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyboardError {
    #[error("{0} is not bound to the virtual keyboard")]
    KeyNotBound(KeyCode),

    #[error("{0} is not a valid key value")]
    InvalidKeyValue(u8),
}
