use rand::{RngExt, random, rngs::ThreadRng};

use crate::{
    display::StandardScreen,
    instructions::{Instructions, InstructionsError},
    keyboard::{Keyboard, KeyboardError},
    memory::{self, Address, Memory, MemoryError},
    registers::{Registers, RegistersError, VRegister},
};

#[derive(Debug, Default)]
pub struct Cpu {
    registers: Registers,
    keyboard: Keyboard,
    memory: Memory,
    screen: StandardScreen,

    rng: ThreadRng,
}

impl Cpu {
    fn new() -> Self {
        Self::default()
    }

    fn skip_instr(&mut self) {
        self.registers.program_counter += 2;
    }

    fn v_reg(&self, reg_index: VRegister) -> u8 {
        self.registers.v_registers[reg_index as usize]
    }

    fn set_v_reg(&mut self, reg_index: VRegister, value: VRegister) {
        self.registers.v_registers[reg_index as usize] = value;
    }

    fn set_pc(&mut self, addr: Address) {
        self.registers.program_counter = addr
    }

    fn set_f(&mut self, value: VRegister) {
        self.set_v_reg(0xF, value);
    }

    pub(crate) fn execute(&mut self, instr: Instructions) -> Result<(), ExecutionError> {
        match instr {
            Instructions::CLS => self.screen.clear(),

            Instructions::RET => {
                let addr = self.registers.pop_stack()?;
                self.set_pc(addr);
            }

            Instructions::JP(addr) => self.set_pc(addr),

            Instructions::CALL(addr) => {
                self.registers.push_stack(self.registers.program_counter)?;
                self.set_pc(addr);
            }

            Instructions::SE_Value(vx, kk) => {
                if self.v_reg(vx) == kk {
                    self.skip_instr();
                }
            }

            Instructions::SNE(vx, kk) => {
                if self.v_reg(vx) != kk {
                    self.skip_instr();
                }
            }

            Instructions::SE_Reg(vx, vy) => {
                if self.v_reg(vx) == self.v_reg(vy) {
                    self.skip_instr();
                }
            }

            Instructions::LD(vx, kk) => self.set_v_reg(vx, kk),

            Instructions::ADD(vx, kk) => self.set_v_reg(vx, self.v_reg(vx) + kk),

            Instructions::LD_Regs(vx, vy) => self.set_v_reg(vx, self.v_reg(vy)),

            Instructions::OR(vx, vy) => self.set_v_reg(vx, self.v_reg(vx) | self.v_reg(vy)),

            Instructions::AND(vx, vy) => self.set_v_reg(vx, self.v_reg(vx) & self.v_reg(vy)),

            Instructions::XOR(vx, vy) => self.set_v_reg(vx, self.v_reg(vx) ^ self.v_reg(vy)),

            Instructions::ADD_Reg(vx, vy) => {
                let (res, carry) = self.v_reg(vx).overflowing_add(self.v_reg(vy));
                self.set_v_reg(vx, res);
                self.set_f(carry.into());
            }

            Instructions::SUB(vx, vy) => {
                let (res, carry) = self.v_reg(vx).overflowing_sub(self.v_reg(vy));
                self.set_v_reg(vx, res);
                self.set_f((!carry).into());
            }

            Instructions::SHR(vx) => {
                let value = self.v_reg(vx);
                self.set_f(value & 1);
                self.set_v_reg(vx, value >> 1);
            }

            Instructions::SUBN(vx, vy) => {
                let (res, carry) = self.v_reg(vy).overflowing_sub(self.v_reg(vx));
                self.set_v_reg(vx, res);
                self.set_f((!carry).into());
            }

            Instructions::SHL(vx) => {
                let value = self.v_reg(vx);
                self.set_f(value & 1);
                self.set_v_reg(vx, value << 1);
            }

            Instructions::SNE_Reg(vx, vy) => {
                if self.v_reg(vx) != self.v_reg(vy) {
                    self.skip_instr();
                }
            }

            Instructions::LD_I(addr) => self.registers.i = addr,

            Instructions::JP_V0(addr) => {
                self.set_pc(self.v_reg(0) as Address + addr);
            }

            Instructions::RND(vx, kk) => {
                let rnd: u8 = self.rng.random();
                self.set_v_reg(vx, rnd & kk);
            }

            Instructions::DRW(vx, vy, n) => {
                let sprite = self.memory.read(self.registers.i, n as Address)?.into();
                let collision = self.screen.write_sprite(
                    &sprite,
                    self.v_reg(vx) as usize,
                    self.v_reg(vy) as usize,
                );
                self.set_f(collision.into());
            }

            Instructions::SKP(vx) => {
                if self.keyboard.is_down(self.v_reg(vx).try_into()?) {
                    self.skip_instr();
                }
            }

            Instructions::SKNP(vx) => {
                if self.keyboard.is_up(self.v_reg(vx).try_into()?) {
                    self.skip_instr();
                }
            }

            Instructions::LD_DT(vx) => self.set_v_reg(vx, self.registers.delay_timer),

            Instructions::LD_K(_) => todo!(),

            Instructions::SET_DT(vx) => self.registers.delay_timer = self.v_reg(vx),

            Instructions::SET_ST(vx) => self.registers.sound_timer = self.v_reg(vx),

            Instructions::ADD_I(vx) => self.registers.i += self.v_reg(vx) as Address,

            Instructions::LD_F(vx) => self.registers.i = memory::digit_addr(self.v_reg(vx)),

            Instructions::LD_B(vx) => {
                let value = self.v_reg(vx);
                self.memory.store(&[value / 100], self.registers.i)?;
                self.memory
                    .store(&[value % 100 / 10], self.registers.i + 1)?;
                self.memory.store(&[value % 10], self.registers.i + 2)?;
            }

            Instructions::LD_MEM_I(vx) => {
                for (value, addr) in self
                    .registers
                    .v_registers
                    .iter()
                    .take(vx as usize + 1)
                    .zip(self.registers.i..)
                {
                    self.memory.store(&[*value], addr)?;
                }
            }

            Instructions::LD_I_MEM(vx) => {
                for (reg, addr) in self
                    .registers
                    .v_registers
                    .iter_mut()
                    .take(vx as usize + 1)
                    .zip(self.registers.i..)
                {
                    *reg = self.memory.read(addr, 1)?[0];
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("register error: {0}")]
    RegistersError(#[from] RegistersError),

    #[error("memory error: {0}")]
    MemoryError(#[from] MemoryError),

    #[error("keyboard error: {0}")]
    KeyboardError(#[from] KeyboardError),
}

#[cfg(test)]
mod tests {
    use crate::{instructions::Instructions, memory::Address};

    use super::Cpu;

    const ADDR: Address = 0x321;

    fn create_interpreter() -> Cpu {
        Cpu::default()
    }

    #[test]
    fn instruction_jp() {
        let mut int = create_interpreter();

        let res = int.execute(Instructions::JP(ADDR));
        assert!(res.is_ok());
        assert_eq!(int.registers.program_counter, ADDR);
    }

    #[test]
    fn instruction_call() {
        let mut int = create_interpreter();

        let pc = int.registers.program_counter;

        int.execute(Instructions::CALL(ADDR)).unwrap();
        assert_eq!(pc, int.registers.top_stack().unwrap());
        assert_eq!(int.registers.program_counter, ADDR);
    }

    #[test]
    fn instruction_se_value() {
        let mut int = create_interpreter();

        int.registers.v_registers[0] = 1;
        let pc = int.registers.program_counter;
        int.execute(Instructions::SE_Value(0, 1)).unwrap();
        assert_eq!(int.registers.program_counter, pc + 2);

        let pc = int.registers.program_counter;
        int.execute(Instructions::SE_Value(0, 2)).unwrap();
        assert_ne!(int.registers.program_counter, pc + 2);
    }

    #[test]
    fn instruction_sne() {
        let mut int = create_interpreter();

        int.registers.v_registers[0] = 0;
        let pc = int.registers.program_counter;
        int.execute(Instructions::SNE(0, 1)).unwrap();
        assert_eq!(int.registers.program_counter, pc + 2);

        let pc = int.registers.program_counter;
        int.execute(Instructions::SNE(0, 0)).unwrap();
        assert_ne!(int.registers.program_counter, pc + 2);
    }

    #[test]
    fn instruction_se_reg() {
        let mut int = create_interpreter();

        let pc = int.registers.program_counter;
        int.registers.v_registers[0] = 1;
        int.registers.v_registers[1] = 1;
        int.execute(Instructions::SE_Reg(0, 1)).unwrap();
        assert_eq!(int.registers.program_counter, pc + 2);

        let pc = int.registers.program_counter;
        int.registers.v_registers[1] = 0;
        int.execute(Instructions::SE_Reg(0, 1)).unwrap();
        assert_ne!(int.registers.program_counter, pc + 2);
    }
}
