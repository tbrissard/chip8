use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Widget},
};

use crate::memory::Address;

/// General purpose register
pub(crate) type VRegister = u8;
pub(super) type TimerValue = u8;

type Result<T> = std::result::Result<T, RegistersError>;

const MAX_SUBROUTINES: u8 = 16;

#[derive(Debug, Default)]
pub(crate) struct Registers {
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
    pub(crate) program_counter: Address,

    /// The stack pointer (SP) can be 8-bit, it is used to point to the topmost level of the stack.
    pub(super) stack_pointer: u8,

    /// The stack is an array of 16 16-bit values, used to store the address that the interpreter shoud return to when finished with a subroutine. Chip-8 allows for up to 16 levels of nested subroutines.
    pub(super) stack: [Address; MAX_SUBROUTINES as usize],
}

impl Registers {
    pub(super) fn push_stack(&mut self, addr: Address) -> Result<()> {
        if self.stack_pointer == MAX_SUBROUTINES {
            return Err(RegistersError::StackFull);
        }

        self.stack[self.stack_pointer as usize] = addr;
        self.stack_pointer += 1;

        Ok(())
    }

    pub(super) fn pop_stack(&mut self) -> Result<Address> {
        if self.stack_pointer == 0 {
            return Err(RegistersError::StackEmpty);
        }

        self.stack_pointer -= 1;
        let addr = self.stack[self.stack_pointer as usize];
        Ok(addr)
    }

    pub(super) fn _top_stack(&self) -> Option<Address> {
        (self.stack_pointer > 0).then_some(self.stack[self.stack_pointer as usize - 1])
    }
}

impl Widget for &Registers {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let title = Line::from("Registers".bold());
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);
        let block_area = block.inner(area);

        let layout = Layout::horizontal(vec![Constraint::Length(8), Constraint::Length(22)])
            .spacing(3)
            .split(block_area);

        let v_registers = Text::from(
            self.v_registers
                .iter()
                .enumerate()
                .map(|(i, vreg)| Line::from(format!("V{i:2}: {vreg:3}")))
                .collect::<Vec<_>>(),
        )
        .centered();

        let mut others = vec![
            Line::from(format!("Program Counter: {:#05X}", self.program_counter)),
            Line::from(format!("I: {:#05X}", self.i)),
            Line::from(""),
            Line::from(format!("Delay Timer: {}", self.delay_timer)),
            Line::from(format!("Sound Timer: {}", self.sound_timer)),
            Line::from(""),
            Line::from(format!("Stack Pointer: {}", self.stack_pointer)),
        ];
        others.extend(
            self.stack
                .iter()
                .map(|addr| format!("{addr:#X}"))
                .map(Line::from),
        );
        let others = Text::from(others);

        v_registers.render(layout[0], buf);
        others.render(layout[1], buf);
        block.render(area, buf);
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
