use crate::{cpu::registers::VRegister, memory::Address};

type Value = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub(super) enum Instructions {
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

    /// Stores the value of register Vy in register Vx.
    LD_Regs(VRegister, VRegister),

    /// Set Vx = Vx OR Vy.
    ///
    /// Performs a bitwise OR on the values of Vx and Vy, then stores the result in Vx. A bitwise OR compares the corrseponding bits from two values, and if either bit is 1, then the same bit in the result is also 1. Otherwise, it is 0.
    OR(VRegister, VRegister),

    /// Set Vx = Vx AND Vy.
    ///
    /// Performs a bitwise AND on the values of Vx and Vy, then stores the result in Vx. A bitwise AND compares the corrseponding bits from two values, and if both bits are 1, then the same bit in the result is also 1. Otherwise, it is 0.
    AND(VRegister, VRegister),

    /// Set Vx = Vx XOR Vy.
    ///
    /// Performs a bitwise exclusive OR on the values of Vx and Vy, then stores the result in Vx. An exclusive OR compares the corrseponding bits from two values, and if the bits are not both the same, then the corresponding bit in the result is set to 1. Otherwise, it is 0.
    XOR(VRegister, VRegister),

    /// Set Vx = Vx + Vy, set VF = carry.
    ///
    /// The values of Vx and Vy are added together. If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0. Only the lowest 8 bits of the result are kept, and stored in Vx.
    ADD_Reg(VRegister, VRegister),

    /// Set Vx = Vx - Vy, set VF = NOT borrow.
    ///
    /// If Vx > Vy, then VF is set to 1, otherwise 0. Then Vy is subtracted from Vx, and the results stored in Vx.
    SUB(VRegister, VRegister),

    /// Set Vx = Vx SHR 1.
    ///
    /// If the least-significant bit of Vx is 1, then VF is set to 1, otherwise 0. Then Vx is divided by 2.
    SHR(VRegister),

    /// Set Vx = Vy - Vx, set VF = NOT borrow.
    ///
    /// If Vy > Vx, then VF is set to 1, otherwise 0. Then Vx is subtracted from Vy, and the results stored in Vx.
    SUBN(VRegister, VRegister),

    /// Set Vx = Vx SHL 1.
    ///
    /// If the most-significant bit of Vx is 1, then VF is set to 1, otherwise to 0. Then Vx is multiplied by 2.
    SHL(VRegister),

    /// Skip next instruction if Vx != Vy.
    ///
    /// The values of Vx and Vy are compared, and if they are not equal, the program counter is increased by 2.
    SNE_Reg(VRegister, VRegister),

    /// Set I = nnn.
    ///
    /// The value of register I is set to nnn.
    LD_I(Address),

    /// Jump to location nnn + V0.
    ///
    /// The program counter is set to nnn plus the value of V0.
    JP_V0(Address),

    /// Set Vx = random byte AND kk.
    ///
    /// The interpreter generates a random number from 0 to 255, which is then ANDed with the value kk. The results are stored in Vx. See instruction 8xy2 for more information on AND.
    RND(VRegister, Value),

    /// Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    ///
    /// The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0. If the sprite is positioned so part of it is outside the coordinates of the display, it wraps around to the opposite side of the screen. See instruction 8xy3 for more information on XOR, and section 2.4, Display, for more information on the Chip-8 screen and sprites.
    DRW(VRegister, VRegister, u8),

    /// Skip next instruction if key with the value of Vx is pressed.
    ///
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the down position, PC is increased by 2.
    SKP(VRegister),

    /// Skip next instruction if key with the value of Vx is not pressed.
    ///
    /// Checks the keyboard, and if the key corresponding to the value of Vx is currently in the up position, PC is increased by 2.
    SKNP(VRegister),

    /// Set Vx = delay timer value.
    ///
    /// The value of DT is placed into Vx.
    LD_DT(VRegister),

    /// Wait for a key press, store the value of the key in Vx.
    ///
    /// All execution stops until a key is pressed, then the value of that key is stored in Vx.
    LD_K(VRegister),

    /// Set delay timer = Vx.
    ///
    /// DT is set equal to the value of Vx.
    SET_DT(VRegister),

    /// Set sound timer = Vx.
    ///
    /// ST is set equal to the value of Vx.
    SET_ST(VRegister),

    /// Set I = I + Vx.
    ///
    /// The values of I and Vx are added, and the results are stored in I.
    ADD_I(VRegister),

    /// Set I = location of sprite for digit Vx.
    ///
    /// The value of I is set to the location for the hexadecimal sprite corresponding to the value of Vx. See section 2.4, Display, for more information on the Chip-8 hexadecimal font.
    LD_F(VRegister),

    /// Store BCD representation of Vx in memory locations I, I+1, and I+2.
    ///
    /// The interpreter takes the decimal value of Vx, and places the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.
    LD_B(VRegister),

    /// Store registers V0 through Vx in memory starting at location I.
    ///
    /// The interpreter copies the values of registers V0 through Vx into memory, starting at the address in I.
    LD_MEM_I(VRegister),

    /// Read registers V0 through Vx from memory starting at location I.
    ///
    /// The interpreter reads values from memory starting at location I into registers V0 through Vx.
    LD_I_MEM(VRegister),
}

fn split_nibbles(value: [u8; 2]) -> (u8, u8, u8, u8) {
    let (a, b) = split_high_low(value[0]);
    let (c, d) = split_high_low(value[1]);
    (a, b, c, d)
}

fn split_high_low(byte: u8) -> (u8, u8) {
    ((byte & 0xF0) >> 4, byte & 0xF)
}

fn from_nibbles(high: u8, low: u8) -> u8 {
    (high << 4) | low
}

fn from_low_12(a: u8, b: u8, c: u8) -> u16 {
    ((a as u16) << 8) | ((b as u16) << 4) | (c as u16)
}

impl TryFrom<&[u8; 2]> for Instructions {
    type Error = InstructionsError;

    fn try_from(value: &[u8; 2]) -> Result<Self, Self::Error> {
        Ok(match split_nibbles(*value) {
            (0x0, 0x0, 0xE, 0x0) => Self::CLS,
            (0x0, 0x0, 0xE, 0xE) => Self::RET,
            (0x1, b, c, d) => Self::JP(from_low_12(b, c, d)),
            (0x2, b, c, d) => Self::CALL(from_low_12(b, c, d)),
            (0x3, b, c, d) => Self::SE_Value(b, from_nibbles(c, d)),
            (0x4, b, c, d) => Self::SNE(b, from_nibbles(c, d)),
            (0x5, b, c, 0x0) => Self::SE_Reg(b, c),
            (0x6, b, c, d) => Self::LD(b, from_nibbles(c, d)),
            (0x7, b, c, d) => Self::ADD(b, from_nibbles(c, d)),
            (0x8, b, c, 0x0) => Self::LD_Regs(b, c),
            (0x8, b, c, 0x1) => Self::OR(b, c),
            (0x8, b, c, 0x2) => Self::AND(b, c),
            (0x8, b, c, 0x3) => Self::XOR(b, c),
            (0x8, b, c, 0x4) => Self::ADD_Reg(b, c),
            (0x8, b, c, 0x5) => Self::SUB(b, c),
            (0x8, b, _, 0x6) => Self::SHR(b),
            (0x8, b, c, 0x7) => Self::SUBN(b, c),
            (0x8, b, _, 0xE) => Self::SHL(b),
            (0x9, b, c, 0x0) => Self::SNE_Reg(b, c),
            (0xA, b, c, d) => Self::LD_I(from_low_12(b, c, d)),
            (0xB, b, c, d) => Self::JP_V0(from_low_12(b, c, d)),
            (0xC, b, c, d) => Self::RND(b, from_nibbles(c, d)),
            (0xD, b, c, d) => Self::DRW(b, c, d),
            (0xE, b, 0x9, 0xE) => Self::SKP(b),
            (0xE, b, 0xA, 0x1) => Self::SKNP(b),
            (0xF, b, 0x0, 0x7) => Self::LD_DT(b),
            (0xF, b, 0x0, 0xA) => Self::LD_K(b),
            (0xF, b, 0x1, 0x5) => Self::SET_DT(b),
            (0xF, b, 0x1, 0x8) => Self::SET_ST(b),
            (0xF, b, 0x1, 0xE) => Self::ADD_I(b),
            (0xF, b, 0x2, 0x9) => Self::LD_F(b),
            (0xF, b, 0x3, 0x3) => Self::LD_B(b),
            (0xF, b, 0x5, 0x5) => Self::LD_MEM_I(b),
            (0xF, b, 0x6, 0x5) => Self::LD_I_MEM(b),
            _ => return Err(InstructionsError::InvalidInstruction(*value)),
        })
    }
}

impl TryFrom<u16> for Instructions {
    type Error = InstructionsError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::try_from(&u16::to_be_bytes(value))
    }
}

impl std::fmt::Display for Instructions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const WIDTH: usize = 20;
        match self {
            Instructions::CLS => write!(f, "{:<WIDTH$}", "CLS"),
            Instructions::RET => write!(f, "{:<WIDTH$}", "RET"),
            Instructions::JP(addr) => write!(f, "{:<WIDTH$}", format!("JP {:#05X}", addr)),
            Instructions::CALL(addr) => write!(f, "{:<WIDTH$}", format!("CALL {:#05X}", addr)),
            Instructions::SE_Value(vx, kk) => {
                write!(f, "{:<WIDTH$}", format!("SE V{}, {}", vx, kk))
            }
            Instructions::SNE(vx, kk) => {
                write!(f, "{:<WIDTH$}", format!("SNE V{}, {}", vx, kk))
            }
            Instructions::SE_Reg(vx, vy) => write!(f, "{:<WIDTH$}", format!("SE V{}, V{}", vx, vy)),
            Instructions::LD(vx, kk) => write!(f, "{:<WIDTH$}", format!("LD V{}, {}", vx, kk)),
            Instructions::ADD(vx, kk) => write!(f, "{:<WIDTH$}", format!("ADD V{}, {}", vx, kk)),
            Instructions::LD_Regs(vx, vy) => {
                write!(f, "{:<WIDTH$}", format!("LD V{}, V{}", vx, vy))
            }
            Instructions::OR(vx, vy) => write!(f, "{:<WIDTH$}", format!("OR V{}, V{}", vx, vy)),
            Instructions::AND(vx, vy) => write!(f, "{:<WIDTH$}", format!("AND V{}, V{}", vx, vy)),
            Instructions::XOR(vx, vy) => write!(f, "{:<WIDTH$}", format!("XOR V{}, V{}", vx, vy)),
            Instructions::ADD_Reg(vx, vy) => {
                write!(f, "{:<WIDTH$}", format!("ADD V{}, V{}", vx, vy))
            }
            Instructions::SUB(vx, vy) => write!(f, "{:<WIDTH$}", format!("SUB V{}, V{}", vx, vy)),
            Instructions::SHR(vx) => write!(f, "{:<WIDTH$}", format!("SHR V{}", vx)),
            Instructions::SUBN(vx, vy) => write!(f, "{:<WIDTH$}", format!("SUBN V{}, V{}", vx, vy)),
            Instructions::SHL(vx) => write!(f, "{:<WIDTH$}", format!("SHL V{}", vx)),
            Instructions::SNE_Reg(vx, vy) => {
                write!(f, "{:<WIDTH$}", format!("SNE V{}, V{}", vx, vy))
            }
            Instructions::LD_I(addr) => write!(f, "{:<WIDTH$}", format!("LD I, {:#05X}", addr)),
            Instructions::JP_V0(addr) => write!(f, "{:<WIDTH$}", format!("JP V0, {:#05X}", addr)),
            Instructions::RND(vx, kk) => write!(f, "{:<WIDTH$}", format!("RND V{}, {}", vx, kk)),
            Instructions::DRW(vx, vy, n) => {
                write!(f, "{:<WIDTH$}", format!("DRW V{}, V{}, {}", vx, vy, n))
            }
            Instructions::SKP(vx) => write!(f, "{:<WIDTH$}", format!("SKP V{}", vx)),
            Instructions::SKNP(vx) => write!(f, "{:<WIDTH$}", format!("SKNP V{}", vx)),
            Instructions::LD_DT(vx) => write!(f, "{:<WIDTH$}", format!("LD V{}, DT", vx)),
            Instructions::LD_K(vx) => write!(f, "{:<WIDTH$}", format!("LD V{}, K", vx)),
            Instructions::SET_DT(vx) => write!(f, "{:<WIDTH$}", format!("LD DT, V{}", vx)),
            Instructions::SET_ST(vx) => write!(f, "{:<WIDTH$}", format!("LD ST, V{}", vx)),
            Instructions::ADD_I(vx) => write!(f, "{:<WIDTH$}", format!("ADD I, V{}", vx)),
            Instructions::LD_F(vx) => write!(f, "{:<WIDTH$}", format!("LD F, V{}", vx)),
            Instructions::LD_B(vx) => write!(f, "{:<WIDTH$}", format!("LD B, V{}", vx)),
            Instructions::LD_MEM_I(vx) => write!(f, "{:<WIDTH$}", format!("LD [I], V{}", vx)),
            Instructions::LD_I_MEM(vx) => write!(f, "{:<WIDTH$}", format!("LD V{}, [I]", vx)),
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InstructionsError {
    #[error("{:#06X} is not a valid instruction", u16::from_be_bytes(*.0))]
    InvalidInstruction([u8; 2]),
}

#[cfg(test)]
mod tests {
    use super::{Instructions, InstructionsError};

    #[test]
    fn test_non_existent_instruction() {
        let instr = Instructions::try_from(0x0000);
        assert_eq!(
            instr,
            Err(InstructionsError::InvalidInstruction([0x00, 0x00]))
        );
    }

    #[test]
    fn test_parsing_cls() {
        assert_eq!(Instructions::try_from(0x00E0), Ok(Instructions::CLS));
    }

    #[test]
    fn test_parsing_ret() {
        assert_eq!(Instructions::try_from(0x00EE), Ok(Instructions::RET));
    }

    #[test]
    fn test_parsing_jp() {
        assert_eq!(Instructions::try_from(0x1200), Ok(Instructions::JP(0x200)))
    }

    #[test]
    fn test_parsing_call() {
        assert_eq!(
            Instructions::try_from(0x2200),
            Ok(Instructions::CALL(0x200))
        )
    }

    #[test]
    fn test_parsing_se() {
        assert_eq!(
            Instructions::try_from(0x3142),
            Ok(Instructions::SE_Value(0x1, 0x42))
        )
    }

    #[test]
    fn test_parsing_sne() {
        assert_eq!(
            Instructions::try_from(0x4142),
            Ok(Instructions::SNE(1, 0x42))
        )
    }

    #[test]
    fn test_parsing_se_reg() {
        assert_eq!(
            Instructions::try_from(0x5120),
            Ok(Instructions::SE_Reg(0x1, 0x2))
        )
    }

    #[test]
    fn test_parsing_ld() {
        assert_eq!(
            Instructions::try_from(0x6142),
            Ok(Instructions::LD(0x1, 0x42))
        )
    }

    #[test]
    fn test_parsing_add() {
        assert_eq!(
            Instructions::try_from(0x7142),
            Ok(Instructions::ADD(0x1, 0x42))
        )
    }

    #[test]
    fn test_parsing_ld_regs() {
        assert_eq!(
            Instructions::try_from(0x8120),
            Ok(Instructions::LD_Regs(0x1, 0x2))
        )
    }

    #[test]
    fn test_parsing_or() {
        assert_eq!(
            Instructions::try_from(0x8121),
            Ok(Instructions::OR(0x1, 0x2))
        )
    }

    #[test]
    fn test_parsing_and() {
        assert_eq!(
            Instructions::try_from(0x8122),
            Ok(Instructions::AND(0x1, 0x2))
        )
    }

    #[test]
    fn test_parsing_xor() {
        assert_eq!(
            Instructions::try_from(0x8123),
            Ok(Instructions::XOR(0x1, 0x2))
        )
    }

    #[test]
    fn test_parsing_add_reg() {
        assert_eq!(
            Instructions::try_from(0x8124),
            Ok(Instructions::ADD_Reg(0x1, 0x2))
        )
    }

    #[test]
    fn test_parsing_sub() {
        assert_eq!(
            Instructions::try_from(0x8125),
            Ok(Instructions::SUB(0x1, 0x2))
        )
    }

    #[test]
    fn test_parsing_shr() {
        assert_eq!(Instructions::try_from(0x8126), Ok(Instructions::SHR(0x1)))
    }

    #[test]
    fn test_parsing_subn() {
        assert_eq!(
            Instructions::try_from(0x8127),
            Ok(Instructions::SUBN(0x1, 0x2))
        )
    }

    #[test]
    fn test_parsing_shl() {
        assert_eq!(Instructions::try_from(0x812E), Ok(Instructions::SHL(0x1)))
    }

    #[test]
    fn test_parsing_sne_reg() {
        assert_eq!(
            Instructions::try_from(0x9120),
            Ok(Instructions::SNE_Reg(0x1, 0x2))
        )
    }

    #[test]
    fn test_parsing_ld_i() {
        assert_eq!(
            Instructions::try_from(0xA200),
            Ok(Instructions::LD_I(0x200))
        )
    }

    #[test]
    fn test_parsing_jp_v0() {
        assert_eq!(
            Instructions::try_from(0xB200),
            Ok(Instructions::JP_V0(0x200))
        )
    }

    #[test]
    fn test_parsing_rnd() {
        assert_eq!(
            Instructions::try_from(0xC142),
            Ok(Instructions::RND(0x1, 0x42))
        )
    }

    #[test]
    fn test_parsing_drw() {
        assert_eq!(
            Instructions::try_from(0xD123),
            Ok(Instructions::DRW(0x1, 0x2, 0x3))
        )
    }

    #[test]
    fn test_parsing_skp() {
        assert_eq!(Instructions::try_from(0xE19E), Ok(Instructions::SKP(0x1)))
    }

    #[test]
    fn test_parsing_sknp() {
        assert_eq!(Instructions::try_from(0xE1A1), Ok(Instructions::SKNP(0x1)))
    }

    #[test]
    fn test_parsing_ld_dt() {
        assert_eq!(Instructions::try_from(0xF107), Ok(Instructions::LD_DT(0x1)))
    }

    #[test]
    fn test_parsing_ld_k() {
        assert_eq!(Instructions::try_from(0xF10A), Ok(Instructions::LD_K(0x1)))
    }

    #[test]
    fn test_parsing_set_dt() {
        assert_eq!(
            Instructions::try_from(0xF115),
            Ok(Instructions::SET_DT(0x1))
        )
    }

    #[test]
    fn test_parsing_set_st() {
        assert_eq!(
            Instructions::try_from(0xF118),
            Ok(Instructions::SET_ST(0x1))
        )
    }

    #[test]
    fn test_parsing_add_i() {
        assert_eq!(Instructions::try_from(0xF11E), Ok(Instructions::ADD_I(0x1)))
    }

    #[test]
    fn test_parsing_ld_f() {
        assert_eq!(Instructions::try_from(0xF129), Ok(Instructions::LD_F(0x1)))
    }

    #[test]
    fn test_parsing_ld_b() {
        assert_eq!(Instructions::try_from(0xF133), Ok(Instructions::LD_B(0x1)))
    }

    #[test]
    fn test_parsing_ld_mem_i() {
        assert_eq!(
            Instructions::try_from(0xF155),
            Ok(Instructions::LD_MEM_I(0x1))
        )
    }

    #[test]
    fn test_parsing_ld_i_mem() {
        assert_eq!(
            Instructions::try_from(0xF165),
            Ok(Instructions::LD_I_MEM(0x1))
        )
    }
}
