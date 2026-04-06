use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Ch8Key {
    Zero,
    One,
    Two,
    Three,
    Four,
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

impl Ch8Key {
    pub(crate) const VARIANTS: [Self; 16] = [
        Self::Zero,
        Self::One,
        Self::Two,
        Self::Three,
        Self::Four,
        Self::Five,
        Self::Six,
        Self::Seven,
        Self::Eight,
        Self::Nine,
        Self::A,
        Self::B,
        Self::C,
        Self::D,
        Self::E,
        Self::F,
    ];
}

impl Display for Ch8Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ch8Key::Zero => write!(f, "0"),
            Ch8Key::One => write!(f, "1"),
            Ch8Key::Two => write!(f, "2"),
            Ch8Key::Three => write!(f, "3"),
            Ch8Key::Four => write!(f, "4"), // Assuming "Fout" is a typo for "Four"
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

impl From<Ch8Key> for u8 {
    fn from(value: Ch8Key) -> Self {
        match value {
            Ch8Key::Zero => 0x0,
            Ch8Key::One => 0x1,
            Ch8Key::Two => 0x2,
            Ch8Key::Three => 0x3,
            Ch8Key::Four => 0x4,
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
    type Error = KeyError;

    fn try_from(value: u8) -> Result<Self, KeyError> {
        Ok(match value {
            0x0 => Ch8Key::Zero,
            0x1 => Ch8Key::One,
            0x2 => Ch8Key::Two,
            0x3 => Ch8Key::Three,
            0x4 => Ch8Key::Four,
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
            _ => return Err(KeyError::InvalidKeyValue(value)),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeyError {
    #[error("{0} is not a valid key value")]
    InvalidKeyValue(u8),
}
