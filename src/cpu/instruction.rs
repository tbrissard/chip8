use crate::{
    cpu::registers::{VRegister, VRegisterValue},
    memory::Address,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum Instruction {
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
    SE_Value(VRegister, VRegisterValue),

    /// Skip next instruction if Vx != kk.
    ///
    /// The interpreter compares register Vx to kk, and if they are not equal, increments the program counter by 2.
    SNE(VRegister, VRegisterValue),

    /// Skip next instruction if Vx = Vy.
    ///
    /// The interpreter compares register Vx to register Vy, and if they are equal, increments the program counter by 2.
    SE_Reg(VRegister, VRegister),

    /// Set Vx = kk.
    ///
    /// The interpreter puts the value kk into register Vx.
    LD(VRegister, VRegisterValue),

    /// Set Vx = Vx + kk.
    ///
    /// Adds the value kk to the value of register Vx, then stores the result in Vx.
    ADD(VRegister, VRegisterValue),

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
    RND(VRegister, VRegisterValue),

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

impl TryFrom<&[u8; 2]> for Instruction {
    type Error = InstructionError;

    fn try_from(value: &[u8; 2]) -> Result<Self, Self::Error> {
        let (a, b, c, d) = split_nibbles(*value);
        let b_reg = VRegister::try_from(b).unwrap();
        let c_reg = VRegister::try_from(c).unwrap();
        Ok(match (a, b, c, d) {
            (0x0, 0x0, 0xE, 0x0) => Self::CLS,
            (0x0, 0x0, 0xE, 0xE) => Self::RET,
            (0x1, b, c, d) => Self::JP(from_low_12(b, c, d)),
            (0x2, b, c, d) => Self::CALL(from_low_12(b, c, d)),
            (0x3, _, c, d) => Self::SE_Value(b_reg, from_nibbles(c, d)),
            (0x4, _, c, d) => Self::SNE(b_reg, from_nibbles(c, d)),
            (0x5, _, _, 0x0) => Self::SE_Reg(b_reg, c_reg),
            (0x6, _, c, d) => Self::LD(b_reg, from_nibbles(c, d)),
            (0x7, _, c, d) => Self::ADD(b_reg, from_nibbles(c, d)),
            (0x8, _, _, 0x0) => Self::LD_Regs(b_reg, c_reg),
            (0x8, _, _, 0x1) => Self::OR(b_reg, c_reg),
            (0x8, _, _, 0x2) => Self::AND(b_reg, c_reg),
            (0x8, _, _, 0x3) => Self::XOR(b_reg, c_reg),
            (0x8, _, _, 0x4) => Self::ADD_Reg(b_reg, c_reg),
            (0x8, _, _, 0x5) => Self::SUB(b_reg, c_reg),
            (0x8, _, _, 0x6) => Self::SHR(b_reg),
            (0x8, _, _, 0x7) => Self::SUBN(b_reg, c_reg),
            (0x8, _, _, 0xE) => Self::SHL(b_reg),
            (0x9, _, _, 0x0) => Self::SNE_Reg(b_reg, c_reg),
            (0xA, b, c, d) => Self::LD_I(from_low_12(b, c, d)),
            (0xB, b, c, d) => Self::JP_V0(from_low_12(b, c, d)),
            (0xC, _, c, d) => Self::RND(b_reg, from_nibbles(c, d)),
            (0xD, _, _, d) => Self::DRW(b_reg, c_reg, d),
            (0xE, _, 0x9, 0xE) => Self::SKP(b_reg),
            (0xE, _, 0xA, 0x1) => Self::SKNP(b_reg),
            (0xF, _, 0x0, 0x7) => Self::LD_DT(b_reg),
            (0xF, _, 0x0, 0xA) => Self::LD_K(b_reg),
            (0xF, _, 0x1, 0x5) => Self::SET_DT(b_reg),
            (0xF, _, 0x1, 0x8) => Self::SET_ST(b_reg),
            (0xF, _, 0x1, 0xE) => Self::ADD_I(b_reg),
            (0xF, _, 0x2, 0x9) => Self::LD_F(b_reg),
            (0xF, _, 0x3, 0x3) => Self::LD_B(b_reg),
            (0xF, _, 0x5, 0x5) => Self::LD_MEM_I(b_reg),
            (0xF, _, 0x6, 0x5) => Self::LD_I_MEM(b_reg),
            _ => return Err(InstructionError::InvalidInstruction(*value)),
        })
    }
}

impl TryFrom<u16> for Instruction {
    type Error = InstructionError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::try_from(&u16::to_be_bytes(value))
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const WIDTH: usize = 20;
        match self {
            Instruction::CLS => write!(f, "{:<WIDTH$}", "CLS"),
            Instruction::RET => write!(f, "{:<WIDTH$}", "RET"),
            Instruction::JP(addr) => write!(f, "{:<WIDTH$}", format!("JP {:#05X}", addr)),
            Instruction::CALL(addr) => write!(f, "{:<WIDTH$}", format!("CALL {:#05X}", addr)),
            Instruction::SE_Value(vx, kk) => {
                write!(f, "{:<WIDTH$}", format!("SE V{}, {}", vx, kk))
            }
            Instruction::SNE(vx, kk) => {
                write!(f, "{:<WIDTH$}", format!("SNE V{}, {}", vx, kk))
            }
            Instruction::SE_Reg(vx, vy) => write!(f, "{:<WIDTH$}", format!("SE V{}, V{}", vx, vy)),
            Instruction::LD(vx, kk) => write!(f, "{:<WIDTH$}", format!("LD V{}, {}", vx, kk)),
            Instruction::ADD(vx, kk) => write!(f, "{:<WIDTH$}", format!("ADD V{}, {}", vx, kk)),
            Instruction::LD_Regs(vx, vy) => {
                write!(f, "{:<WIDTH$}", format!("LD V{}, V{}", vx, vy))
            }
            Instruction::OR(vx, vy) => write!(f, "{:<WIDTH$}", format!("OR V{}, V{}", vx, vy)),
            Instruction::AND(vx, vy) => write!(f, "{:<WIDTH$}", format!("AND V{}, V{}", vx, vy)),
            Instruction::XOR(vx, vy) => write!(f, "{:<WIDTH$}", format!("XOR V{}, V{}", vx, vy)),
            Instruction::ADD_Reg(vx, vy) => {
                write!(f, "{:<WIDTH$}", format!("ADD V{}, V{}", vx, vy))
            }
            Instruction::SUB(vx, vy) => write!(f, "{:<WIDTH$}", format!("SUB V{}, V{}", vx, vy)),
            Instruction::SHR(vx) => write!(f, "{:<WIDTH$}", format!("SHR V{}", vx)),
            Instruction::SUBN(vx, vy) => write!(f, "{:<WIDTH$}", format!("SUBN V{}, V{}", vx, vy)),
            Instruction::SHL(vx) => write!(f, "{:<WIDTH$}", format!("SHL V{}", vx)),
            Instruction::SNE_Reg(vx, vy) => {
                write!(f, "{:<WIDTH$}", format!("SNE V{}, V{}", vx, vy))
            }
            Instruction::LD_I(addr) => write!(f, "{:<WIDTH$}", format!("LD I, {:#05X}", addr)),
            Instruction::JP_V0(addr) => write!(f, "{:<WIDTH$}", format!("JP V0, {:#05X}", addr)),
            Instruction::RND(vx, kk) => write!(f, "{:<WIDTH$}", format!("RND V{}, {}", vx, kk)),
            Instruction::DRW(vx, vy, n) => {
                write!(f, "{:<WIDTH$}", format!("DRW V{}, V{}, {}", vx, vy, n))
            }
            Instruction::SKP(vx) => write!(f, "{:<WIDTH$}", format!("SKP V{}", vx)),
            Instruction::SKNP(vx) => write!(f, "{:<WIDTH$}", format!("SKNP V{}", vx)),
            Instruction::LD_DT(vx) => write!(f, "{:<WIDTH$}", format!("LD V{}, DT", vx)),
            Instruction::LD_K(vx) => write!(f, "{:<WIDTH$}", format!("LD V{}, KEY", vx)),
            Instruction::SET_DT(vx) => write!(f, "{:<WIDTH$}", format!("LD DT, V{}", vx)),
            Instruction::SET_ST(vx) => write!(f, "{:<WIDTH$}", format!("LD ST, V{}", vx)),
            Instruction::ADD_I(vx) => write!(f, "{:<WIDTH$}", format!("ADD I, V{}", vx)),
            Instruction::LD_F(vx) => write!(f, "{:<WIDTH$}", format!("LD F, V{}", vx)),
            Instruction::LD_B(vx) => write!(f, "{:<WIDTH$}", format!("LD B, V{}", vx)),
            Instruction::LD_MEM_I(vx) => write!(f, "{:<WIDTH$}", format!("LD [I], V{}", vx)),
            Instruction::LD_I_MEM(vx) => write!(f, "{:<WIDTH$}", format!("LD V{}, [I]", vx)),
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InstructionError {
    #[error("{:#06X} is not a valid instruction", u16::from_be_bytes(*.0))]
    InvalidInstruction([u8; 2]),
}

#[cfg(test)]
mod tests {

    use super::*;

    const REG_1: VRegister = VRegister::V1;
    const REG_2: VRegister = VRegister::V2;
    const ADDR: Address = 0x200;

    #[test]
    fn test_non_existent_instruction() {
        let instr = Instruction::try_from(0x0000);
        assert_eq!(
            instr,
            Err(InstructionError::InvalidInstruction([0x00, 0x00]))
        );
    }

    #[test]
    fn test_parsing_cls() {
        assert_eq!(Instruction::try_from(0x00E0), Ok(Instruction::CLS));
    }

    #[test]
    fn test_parsing_ret() {
        assert_eq!(Instruction::try_from(0x00EE), Ok(Instruction::RET));
    }

    #[test]
    fn test_parsing_jp() {
        assert_eq!(Instruction::try_from(0x1200), Ok(Instruction::JP(ADDR)))
    }

    #[test]
    fn test_parsing_call() {
        assert_eq!(Instruction::try_from(0x2200), Ok(Instruction::CALL(ADDR)))
    }

    #[test]
    fn test_parsing_se() {
        assert_eq!(
            Instruction::try_from(0x3142),
            Ok(Instruction::SE_Value(REG_1, 0x42))
        )
    }

    #[test]
    fn test_parsing_sne() {
        assert_eq!(
            Instruction::try_from(0x4142),
            Ok(Instruction::SNE(REG_1, 0x42))
        )
    }

    #[test]
    fn test_parsing_se_reg() {
        assert_eq!(
            Instruction::try_from(0x5120),
            Ok(Instruction::SE_Reg(REG_1, REG_2))
        )
    }

    #[test]
    fn test_parsing_ld() {
        assert_eq!(
            Instruction::try_from(0x6142),
            Ok(Instruction::LD(REG_1, 0x42))
        )
    }

    #[test]
    fn test_parsing_add() {
        assert_eq!(
            Instruction::try_from(0x7142),
            Ok(Instruction::ADD(REG_1, 0x42))
        )
    }

    #[test]
    fn test_parsing_ld_regs() {
        assert_eq!(
            Instruction::try_from(0x8120),
            Ok(Instruction::LD_Regs(REG_1, REG_2))
        )
    }

    #[test]
    fn test_parsing_or() {
        assert_eq!(
            Instruction::try_from(0x8121),
            Ok(Instruction::OR(REG_1, REG_2))
        )
    }

    #[test]
    fn test_parsing_and() {
        assert_eq!(
            Instruction::try_from(0x8122),
            Ok(Instruction::AND(REG_1, REG_2))
        )
    }

    #[test]
    fn test_parsing_xor() {
        assert_eq!(
            Instruction::try_from(0x8123),
            Ok(Instruction::XOR(REG_1, REG_2))
        )
    }

    #[test]
    fn test_parsing_add_reg() {
        assert_eq!(
            Instruction::try_from(0x8124),
            Ok(Instruction::ADD_Reg(REG_1, REG_2))
        )
    }

    #[test]
    fn test_parsing_sub() {
        assert_eq!(
            Instruction::try_from(0x8125),
            Ok(Instruction::SUB(REG_1, REG_2))
        )
    }

    #[test]
    fn test_parsing_shr() {
        assert_eq!(Instruction::try_from(0x8126), Ok(Instruction::SHR(REG_1)))
    }

    #[test]
    fn test_parsing_subn() {
        assert_eq!(
            Instruction::try_from(0x8127),
            Ok(Instruction::SUBN(REG_1, REG_2))
        )
    }

    #[test]
    fn test_parsing_shl() {
        assert_eq!(Instruction::try_from(0x812E), Ok(Instruction::SHL(REG_1)))
    }

    #[test]
    fn test_parsing_sne_reg() {
        assert_eq!(
            Instruction::try_from(0x9120),
            Ok(Instruction::SNE_Reg(REG_1, REG_2))
        )
    }

    #[test]
    fn test_parsing_ld_i() {
        assert_eq!(Instruction::try_from(0xA200), Ok(Instruction::LD_I(ADDR)))
    }

    #[test]
    fn test_parsing_jp_v0() {
        assert_eq!(Instruction::try_from(0xB200), Ok(Instruction::JP_V0(ADDR)))
    }

    #[test]
    fn test_parsing_rnd() {
        assert_eq!(
            Instruction::try_from(0xC142),
            Ok(Instruction::RND(REG_1, 0x42))
        )
    }

    #[test]
    fn test_parsing_drw() {
        assert_eq!(
            Instruction::try_from(0xD123),
            Ok(Instruction::DRW(REG_1, REG_2, 0x3))
        )
    }

    #[test]
    fn test_parsing_skp() {
        assert_eq!(Instruction::try_from(0xE19E), Ok(Instruction::SKP(REG_1)))
    }

    #[test]
    fn test_parsing_sknp() {
        assert_eq!(Instruction::try_from(0xE1A1), Ok(Instruction::SKNP(REG_1)))
    }

    #[test]
    fn test_parsing_ld_dt() {
        assert_eq!(Instruction::try_from(0xF107), Ok(Instruction::LD_DT(REG_1)))
    }

    #[test]
    fn test_parsing_ld_k() {
        assert_eq!(Instruction::try_from(0xF10A), Ok(Instruction::LD_K(REG_1)))
    }

    #[test]
    fn test_parsing_set_dt() {
        assert_eq!(
            Instruction::try_from(0xF115),
            Ok(Instruction::SET_DT(REG_1))
        )
    }

    #[test]
    fn test_parsing_set_st() {
        assert_eq!(
            Instruction::try_from(0xF118),
            Ok(Instruction::SET_ST(REG_1))
        )
    }

    #[test]
    fn test_parsing_add_i() {
        assert_eq!(Instruction::try_from(0xF11E), Ok(Instruction::ADD_I(REG_1)))
    }

    #[test]
    fn test_parsing_ld_f() {
        assert_eq!(Instruction::try_from(0xF129), Ok(Instruction::LD_F(REG_1)))
    }

    #[test]
    fn test_parsing_ld_b() {
        assert_eq!(Instruction::try_from(0xF133), Ok(Instruction::LD_B(REG_1)))
    }

    #[test]
    fn test_parsing_ld_mem_i() {
        assert_eq!(
            Instruction::try_from(0xF155),
            Ok(Instruction::LD_MEM_I(REG_1))
        )
    }

    #[test]
    fn test_parsing_ld_i_mem() {
        assert_eq!(
            Instruction::try_from(0xF165),
            Ok(Instruction::LD_I_MEM(REG_1))
        )
    }
}
