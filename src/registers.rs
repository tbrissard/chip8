use crate::memory::Address;

/// General purpose register
type VRegister = u8;

type Result<T> = std::result::Result<T, RegistersError>;

const MAX_SUBROUTINES: u8 = 16;

#[derive(Debug, Default)]
pub(crate) struct Registers {
    /// Chip-8 has 16 general purpose 8-bit registers, usually referred to as Vx, where x is a hexadecimal digit (0 through F).
    /// The VF register should not be used by any program, as it is used as a flag by some instructions.
    v_registers: [VRegister; 16],

    /// This register is generally used to store memory addresses, so only the lowest (rightmost) 12 bits are usually used.
    i_register: Address,

    /// Chip-8 also has two special purpose 8-bit registers, for the delay and sound timers. When these registers are non-zero, they are automatically decremented at a rate of 60Hz.
    delay_timer_register: u8,
    sound_timer_register: u8,

    // Not accessible by programs
    /// used to store the currently executing address
    program_counter: Address,

    /// The stack pointer (SP) can be 8-bit, it is used to point to the topmost level of the stack.
    stack_pointer: u8,

    /// The stack is an array of 16 16-bit values, used to store the address that the interpreter shoud return to when finished with a subroutine. Chip-8 allows for up to 16 levels of nested subroutines.
    stack: [Address; MAX_SUBROUTINES as usize],
}

impl Registers {
    pub(crate) fn push_stack(&mut self, addr: Address) -> Result<()> {
        if self.stack_pointer == MAX_SUBROUTINES {
            return Err(RegistersError::StackFull);
        }

        self.stack[self.stack_pointer as usize] = addr;
        self.stack_pointer += 1;

        Ok(())
    }

    pub(crate) fn pop_stack(&mut self) -> Result<Address> {
        if self.stack_pointer == 0 {
            return Err(RegistersError::StackEmpty);
        }

        let addr = self.stack[self.stack_pointer as usize - 1];
        self.stack_pointer -= 1;
        Ok(addr)
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
    fn test_stack() {
        let mut regs = create_registers();

        assert_eq!(regs.pop_stack(), Err(RegistersError::StackEmpty));
        assert_eq!(regs.push_stack(0x200), Ok(()));
        assert_eq!(regs.pop_stack(), Ok(0x200));
    }

    #[test]
    fn test_stack_overflow() {
        let mut regs = create_registers();

        for _ in 0..16 {
            assert_eq!(regs.push_stack(0x200), Ok(()));
        }
        assert_eq!(regs.push_stack(0x200), Err(RegistersError::StackFull));
    }
}
