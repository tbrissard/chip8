use crate::memory::Address;

/// General purpose register
pub(crate) type VRegister = u8;
pub(crate) type TimerValue = u8;

type Result<T> = std::result::Result<T, RegistersError>;

const MAX_SUBROUTINES: u8 = 16;

#[derive(Debug, Default)]
pub(super) struct Registers {
    /// Chip-8 has 16 general purpose 8-bit registers, usually referred to as Vx, where x is a hexadecimal digit (0 through F).
    /// The VF register should not be used by any program, as it is used as a flag by some instructions.
    pub(super) v_registers: [VRegister; 16],

    /// This register is generally used to store memory addresses, so only the lowest (rightmost) 12 bits are usually used.
    pub(super) i: Address,

    /// Chip-8 also has two special purpose 8-bit registers, for the delay and sound timers. When these registers are non-zero, they are automatically decremented at a rate of 60Hz.
    pub(super) delay_timer: TimerValue,
    pub(super) sound_timer: TimerValue,

    // Not accessible by programs
    /// used to store the currently executing address
    pub(super) program_counter: Address,

    /// The stack pointer (SP) can be 8-bit, it is used to point to the topmost level of the stack.
    pub(super) stack_pointer: u8,

    /// The stack is an array of 16 16-bit values, used to store the address that the interpreter shoud return to when finished with a subroutine. Chip-8 allows for up to 16 levels of nested subroutines.
    pub(super) stack: [Address; MAX_SUBROUTINES as usize],
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

        self.stack_pointer -= 1;
        let addr = self.stack[self.stack_pointer as usize];
        Ok(addr)
    }

    pub(crate) fn top_stack(&self) -> Option<Address> {
        (self.stack_pointer > 0).then_some(self.stack[self.stack_pointer as usize - 1])
    }

    pub(crate) fn decrease_delay_timer(&mut self) {
        self.delay_timer = self.delay_timer.saturating_sub(1);
    }

    pub(crate) fn decrease_sound_timer(&mut self) {
        self.sound_timer = self.sound_timer.saturating_sub(1);
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
        assert_eq!(regs.top_stack(), Some(0x200));
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

    #[test]
    fn timers_underflow() {
        let mut regs = create_registers();

        assert_eq!(regs.delay_timer, 0);
        assert_eq!(regs.sound_timer, 0);

        regs.decrease_delay_timer();
        regs.decrease_sound_timer();

        assert_eq!(regs.delay_timer, 0);
        assert_eq!(regs.sound_timer, 0);
    }
}
