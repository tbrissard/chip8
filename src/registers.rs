use crate::memory::Address;

/// General purpose register
type VRegister = u8;

#[derive(Debug)]
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

    stack: [Address; 16],
}
