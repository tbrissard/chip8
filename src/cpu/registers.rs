use std::{
    fmt::Display,
    ops::{Index, IndexMut},
};

use crate::memory::Address;

/// General purpose register
pub(crate) type VRegisterValue = u8;
pub(super) type TimerValue = u8;

const MAX_SUBROUTINES: u8 = 16;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum VRegister {
    V0,
    V1,
    V2,
    V3,
    V4,
    V5,
    V6,
    V7,
    V8,
    V9,
    VA,
    VB,
    VC,
    VD,
    VE,
    VF,
}

impl Display for VRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Index<VRegister> for Registers {
    type Output = VRegisterValue;
    fn index(&self, index: VRegister) -> &Self::Output {
        &self.v_registers[index as usize]
    }
}

impl IndexMut<VRegister> for Registers {
    fn index_mut(&mut self, index: VRegister) -> &mut Self::Output {
        &mut self.v_registers[index as usize]
    }
}

impl TryFrom<u8> for VRegister {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0x0 => Self::V0,
            0x1 => Self::V1,
            0x2 => Self::V2,
            0x3 => Self::V3,
            0x4 => Self::V4,
            0x5 => Self::V5,
            0x6 => Self::V6,
            0x7 => Self::V7,
            0x8 => Self::V8,
            0x9 => Self::V9,
            0xA => Self::VA,
            0xB => Self::VB,
            0xC => Self::VC,
            0xD => Self::VD,
            0xE => Self::VE,
            0xF => Self::VF,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Default)]
pub(crate) struct Registers {
    /// Chip-8 has 16 general purpose 8-bit registers, usually referred to as Vx, where x is a hexadecimal digit (0 through F).
    /// The VF register should not be used by any program, as it is used as a flag by some instructions.
    pub(crate) v_registers: [VRegisterValue; 16],

    /// This register is generally used to store memory addresses, so only the lowest (rightmost) 12 bits are usually used.
    pub(crate) i: Address,

    /// Chip-8 also has two special purpose 8-bit registers, for the delay and sound timers. When these registers are non-zero, they are automatically decremented at a rate of 60Hz.
    pub(crate) delay_timer: TimerValue,
    pub(crate) sound_timer: TimerValue,

    // Not accessible by programs
    /// used to store the currently executing address
    pub(crate) program_counter: Address,

    /// The stack pointer (SP) can be 8-bit, it is used to point to the topmost level of the stack.
    pub(crate) stack_pointer: u8,

    /// The stack is an array of 16 16-bit values, used to store the address that the interpreter shoud return to when finished with a subroutine. Chip-8 allows for up to 16 levels of nested subroutines.
    pub(crate) stack: [Address; MAX_SUBROUTINES as usize],
}

impl Registers {
    pub(super) fn push_stack(&mut self, addr: Address) -> Result<(), RegistersError> {
        if self.stack_pointer == MAX_SUBROUTINES {
            return Err(RegistersError::StackFull);
        }

        self.stack[self.stack_pointer as usize] = addr;
        self.stack_pointer += 1;

        Ok(())
    }

    pub(super) fn pop_stack(&mut self) -> Result<Address, RegistersError> {
        if self.stack_pointer == 0 {
            return Err(RegistersError::StackEmpty);
        }

        self.stack_pointer -= 1;
        let addr = self.stack[self.stack_pointer as usize];
        Ok(addr)
    }

    pub(crate) fn set_vreg(&mut self, vreg: VRegister, value: VRegisterValue) {
        self[vreg] = value;
    }

    pub(crate) fn vreg(&self, vreg: VRegister) -> VRegisterValue {
        self[vreg]
    }

    pub(super) fn _top_stack(&self) -> Option<Address> {
        (self.stack_pointer > 0).then_some(self.stack[self.stack_pointer as usize - 1])
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RegistersError {
    #[error("maximum nested subroutines reached")]
    StackFull,
    #[error("stack is empty")]
    StackEmpty,
}

#[cfg(test)]
mod tests {
    use super::{Registers, RegistersError};

    fn create_registers() -> Registers {
        Registers::default()
    }

    #[test]
    fn stack() {
        let mut regs = create_registers();

        assert_eq!(regs.pop_stack(), Err(RegistersError::StackEmpty));
        assert_eq!(regs.push_stack(0x200), Ok(()));
        assert_eq!(regs._top_stack(), Some(0x200));
        assert_eq!(regs.pop_stack(), Ok(0x200));
    }

    #[test]
    fn stack_overflow() {
        let mut regs = create_registers();

        for _ in 0..16 {
            assert_eq!(regs.push_stack(0x200), Ok(()));
        }
        assert_eq!(regs.push_stack(0x200), Err(RegistersError::StackFull));
    }
}
