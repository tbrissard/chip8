use crate::{memory::Address, registers::VRegister};

type Value = u8;

#[derive(Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub(crate) enum Instructions {
    /// Clear the display.
    CLS,

    /// Return from a subroutine.
    ///
    /// The interpreter sets the program counter to the address at the top of the stack, then subtracts 1 from the stack pointer.
    RET,

    /// Jump to location nnn.
    ///
    /// The interpreter sets the program counter to nnn.
    JP(Address),

    /// Call subroutine at nnn.
    ///
    /// The interpreter increments the stack pointer, then puts the current PC on the top of the stack. The PC is then set to nnn.
    CALL(Address),

    /// Skip next instruction if Vx = kk.
    ///
    /// The interpreter compares register Vx to kk, and if they are equal, increments the program counter by 2.
    SE_Value(VRegister, Value),

    /// Skip next instruction if Vx != kk.
    ///
    /// The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    SNE(VRegister, Value),

    /// Skip next instruction if Vx = Vy.
    ///
    /// The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    SE_Reg(VRegister, VRegister),

    /// Set Vx = kk.
    ///
    /// The interpreter puts the value kk into register Vx.
    LD(VRegister, Value),

    /// Set Vx = Vx + kk.
    ///
    /// Adds the value kk to the value of register Vx, then stores the result in Vx.
    ADD(VRegister, Value),
}

fn upper_4(value: u16) -> u8 {
    (value >> 12).try_into().unwrap()
}

fn split_4_4_8(value: u16) -> (u8, u8, u8) {
    (
        (value >> 12).try_into().unwrap(),
        ((value & 0xF00) >> 8).try_into().unwrap(),
        (value & 0xFF).try_into().unwrap(),
    )
}

fn split_4_4_4_4(value: u16) -> (u16, u16, u16, u16) {
    (
        value >> 12,
        ((value & 0xF00) >> 8),
        (value & 0xF0) >> 4,
        value & 0xF,
    )
}

impl TryFrom<u16> for Instructions {
    type Error = InstructionsError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(match split_4_4_4_4(value) {
            (0x0, 0x0, 0xE, 0x0) => Self::CLS,
            (0x0, 0x0, 0xE, 0xE) => Self::RET,
            (0x1, b, c, d) => Self::JP((b << 8) + (c << 4) + d),
            (0x2, b, c, d) => Self::CALL((b << 8) + (c << 4) + d),
            (0x3, b, c, d) => Self::SE_Value(b as u8, ((c << 4) + d) as u8),
            (0x4, b, c, d) => Self::SNE(b as u8, ((c << 4) + d) as u8),
            (0x5, b, c, 0x0) => Self::SE_Reg(b as u8, c as u8),
            (0x6, b, c, d) => Self::LD(b as u8, ((c << 4) + d) as u8),
            (0x7, b, c, d) => Self::ADD(b as u8, ((c << 4) + d) as u8),
            _ => return Err(InstructionsError::InvalidInstruction(value)),
        })
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InstructionsError {
    #[error("{0:X} is not a valid instruction")]
    InvalidInstruction(u16),
}

#[cfg(test)]
mod tests {
    use super::{Instructions, InstructionsError};

    #[test]
    fn test_non_existent_instruction() {
        let instr = Instructions::try_from(0x0000);
        assert_eq!(instr, Err(InstructionsError::InvalidInstruction(0x0000)));
    }

    #[test]
    fn test_cls() {
        assert_eq!(Instructions::try_from(0x00E0), Ok(Instructions::CLS));
    }

    #[test]
    fn test_ret() {
        assert_eq!(Instructions::try_from(0x00EE), Ok(Instructions::RET));
    }

    #[test]
    fn test_jp() {
        assert_eq!(Instructions::try_from(0x1200), Ok(Instructions::JP(0x200)))
    }

    #[test]
    fn test_call() {
        assert_eq!(
            Instructions::try_from(0x2200),
            Ok(Instructions::CALL(0x200))
        )
    }

    #[test]
    fn test_se() {
        assert_eq!(
            Instructions::try_from(0x3142),
            Ok(Instructions::SE_Value(0x1, 0x42))
        )
    }

    #[test]
    fn test_sne() {
        assert_eq!(
            Instructions::try_from(0x4142),
            Ok(Instructions::SNE(1, 0x42))
        )
    }

    #[test]
    fn test_se_reg() {
        assert_eq!(
            Instructions::try_from(0x5120),
            Ok(Instructions::SE_Reg(0x1, 0x2))
        )
    }

    #[test]
    fn test_ld() {
        assert_eq!(
            Instructions::try_from(0x6142),
            Ok(Instructions::LD(0x1, 0x42))
        )
    }

    #[test]
    fn test_add() {
        assert_eq!(
            Instructions::try_from(0x7142),
            Ok(Instructions::ADD(0x1, 0x42))
        )
    }
}
