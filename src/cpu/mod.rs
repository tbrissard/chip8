use rand::{RngExt, rngs::ThreadRng};

pub(crate) use crate::cpu::instruction::{Instruction, InstructionError};
pub(crate) use crate::cpu::registers::Registers;
pub(crate) use crate::cpu::registers::VRegister;
pub(crate) use crate::cpu::registers::VRegisterValue;
pub(crate) use crate::memory::MemoryError;
use crate::{
    cpu::registers::RegistersError,
    keyboard::{Ch8Keyboard, KeyError},
    memory::{self, Address, Memory},
    screen::StandardScreen,
};

mod instruction;
mod registers;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ExecutionResult {
    Continue,
    WaitForKey(VRegister),
}

#[derive(Debug)]
pub struct Cpu {
    pub(crate) registers: Registers,
    pub(crate) keyboard: Ch8Keyboard,
    memory: Memory,
    pub(crate) screen: StandardScreen,

    rng: ThreadRng,
}

const START_ADDRESS: Address = 0x200;

impl Default for Cpu {
    fn default() -> Self {
        let registers = Registers {
            program_counter: START_ADDRESS,
            ..Default::default()
        };

        Self {
            registers,
            keyboard: Ch8Keyboard::new(),
            memory: Memory::default(),
            screen: StandardScreen::new(),

            rng: ThreadRng::default(),
        }
    }
}

impl Cpu {
    pub(crate) fn load_program(bytes: &[u8]) -> Result<Self, MemoryError> {
        let mut cpu = Self::default();
        cpu.memory.store(bytes, START_ADDRESS)?;
        Ok(cpu)
    }

    pub(crate) fn next_instr(&mut self) -> Result<Instruction, InstructionFetchError> {
        let a = self.memory.read(self.registers.program_counter, 2)?;
        let a = <&[u8; 2]>::try_from(a).unwrap();
        let instr = std::convert::TryInto::<Instruction>::try_into(a)?;
        self.registers.program_counter += 2;
        Ok(instr)
    }

    pub(crate) fn execute(
        &mut self,
        instr: Instruction,
    ) -> Result<ExecutionResult, ExecutionError> {
        match instr {
            Instruction::CLS => self.screen.clear(),

            Instruction::RET => {
                let addr = self.registers.pop_stack()?;
                self.set_pc(addr);
            }

            Instruction::JP(addr) => self.set_pc(addr),

            Instruction::CALL(addr) => {
                self.registers.push_stack(self.registers.program_counter)?;
                self.set_pc(addr);
            }

            Instruction::SE_Value(vx, kk) => {
                if self.registers.vreg(vx) == kk {
                    self.skip_instr();
                }
            }

            Instruction::SNE(vx, kk) => {
                if self.registers.vreg(vx) != kk {
                    self.skip_instr();
                }
            }

            Instruction::SE_Reg(vx, vy) => {
                if self.vreg(vx) == self.vreg(vy) {
                    self.skip_instr();
                }
            }

            Instruction::LD(vx, kk) => self.set_vreg(vx, kk),

            Instruction::ADD(vx, kk) => self.set_vreg(vx, self.vreg(vx).wrapping_add(kk)),

            Instruction::LD_Regs(vx, vy) => self.set_vreg(vx, self.vreg(vy)),

            Instruction::OR(vx, vy) => self.set_vreg(vx, self.vreg(vx) | self.vreg(vy)),

            Instruction::AND(vx, vy) => self.set_vreg(vx, self.vreg(vx) & self.vreg(vy)),

            Instruction::XOR(vx, vy) => self.set_vreg(vx, self.vreg(vx) ^ self.vreg(vy)),

            Instruction::ADD_Reg(vx, vy) => {
                let (res, carry) = self.vreg(vx).overflowing_add(self.vreg(vy));
                self.set_vreg(vx, res);
                self.set_f(carry.into());
            }

            Instruction::SUB(vx, vy) => {
                let (res, carry) = self.vreg(vx).overflowing_sub(self.vreg(vy));
                self.set_vreg(vx, res);
                self.set_f((!carry).into());
            }

            Instruction::SHR(vx) => {
                let value = self.vreg(vx);
                self.set_f(value & 1);
                self.set_vreg(vx, value >> 1);
            }

            Instruction::SUBN(vx, vy) => {
                let (res, carry) = self.vreg(vy).overflowing_sub(self.vreg(vx));
                self.set_vreg(vx, res);
                self.set_f((!carry).into());
            }

            Instruction::SHL(vx) => {
                let value = self.vreg(vx);
                self.set_f(value & 1);
                self.set_vreg(vx, value << 1);
            }

            Instruction::SNE_Reg(vx, vy) => {
                if self.vreg(vx) != self.vreg(vy) {
                    self.skip_instr();
                }
            }

            Instruction::LD_I(addr) => self.registers.i = addr,

            Instruction::JP_V0(addr) => {
                self.set_pc(self.vreg(VRegister::V0) as Address + addr);
            }

            Instruction::RND(vx, kk) => {
                let rnd: u8 = self.rng.random();
                self.set_vreg(vx, rnd & kk);
            }

            Instruction::DRW(vx, vy, n) => {
                let sprite = self.memory.read(self.registers.i, n as Address)?.into();
                let collision = self.screen.write_sprite(
                    &sprite,
                    self.vreg(vx) as usize,
                    self.vreg(vy) as usize,
                );
                self.set_f(collision.into());
            }

            Instruction::SKP(vx) => {
                if self.keyboard.is_down(self.vreg(vx).try_into()?) {
                    self.skip_instr();
                }
            }

            Instruction::SKNP(vx) => {
                if self.keyboard.is_up(self.vreg(vx).try_into()?) {
                    self.skip_instr();
                }
            }

            Instruction::LD_DT(vx) => self.set_vreg(vx, self.registers.delay_timer),

            Instruction::LD_K(vx) => return Ok(ExecutionResult::WaitForKey(vx)),

            Instruction::SET_DT(vx) => self.registers.delay_timer = self.vreg(vx),

            Instruction::SET_ST(vx) => self.registers.sound_timer = self.vreg(vx),

            Instruction::ADD_I(vx) => self.registers.i += self.vreg(vx) as Address,

            Instruction::LD_F(vx) => self.registers.i = memory::digit_addr(self.vreg(vx)),

            Instruction::LD_B(vx) => {
                let value = self.vreg(vx);
                self.memory.store(&[value / 100], self.registers.i)?;
                self.memory
                    .store(&[value % 100 / 10], self.registers.i + 1)?;
                self.memory.store(&[value % 10], self.registers.i + 2)?;
            }

            Instruction::LD_MEM_I(vx) => {
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

            Instruction::LD_I_MEM(vx) => {
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

        Ok(ExecutionResult::Continue)
    }

    fn skip_instr(&mut self) {
        self.registers.program_counter += 2;
    }

    fn vreg(&self, vreg: VRegister) -> VRegisterValue {
        self.registers.vreg(vreg)
    }

    pub(crate) fn set_vreg(&mut self, vreg: VRegister, value: VRegisterValue) {
        self.registers.set_vreg(vreg, value);
    }

    fn set_f(&mut self, value: VRegisterValue) {
        self.set_vreg(VRegister::VF, value);
    }

    fn set_pc(&mut self, addr: Address) {
        self.registers.program_counter = addr
    }

    pub(crate) fn decrease_delay_timer(&mut self) {
        self.registers.delay_timer = self.registers.delay_timer.saturating_sub(1);
    }

    pub(crate) fn decrease_sound_timer(&mut self) {
        self.registers.sound_timer = self.registers.sound_timer.saturating_sub(1);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InstructionFetchError {
    #[error("{0}")]
    BadInstruction(#[from] InstructionError),

    #[error("{0}")]
    BadMemoryAccess(#[from] MemoryError),
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("register error: {0}")]
    Registers(#[from] RegistersError),

    #[error("memory error: {0}")]
    Memory(#[from] MemoryError),

    #[error(" {0}")]
    BadKeyValue(#[from] KeyError),
}

#[cfg(test)]
mod tests {
    use super::*;

    const REG_1: VRegister = VRegister::V1;
    const REG_2: VRegister = VRegister::V2;
    const ADDR: Address = 0x321;

    fn create_cpu() -> Cpu {
        Cpu::default()
    }

    #[test]
    fn instruction_jp() {
        let mut int = create_cpu();

        let res = int.execute(Instruction::JP(ADDR));
        assert!(res.is_ok());
        assert_eq!(int.registers.program_counter, ADDR);
    }

    #[test]
    fn instruction_call() {
        let mut int = create_cpu();

        let pc = int.registers.program_counter;

        int.execute(Instruction::CALL(ADDR)).unwrap();
        assert_eq!(pc, int.registers._top_stack().unwrap());
        assert_eq!(int.registers.program_counter, ADDR);
    }

    #[test]
    fn instruction_se_value() {
        let mut int = create_cpu();

        int.set_vreg(REG_1, 1);
        let pc = int.registers.program_counter;
        int.execute(Instruction::SE_Value(REG_1, 1)).unwrap();
        assert_eq!(int.registers.program_counter, pc + 2);

        let pc = int.registers.program_counter;
        int.execute(Instruction::SE_Value(REG_1, 2)).unwrap();
        assert_ne!(int.registers.program_counter, pc + 2);
    }

    #[test]
    fn instruction_sne() {
        let mut int = create_cpu();

        int.set_vreg(REG_1, 0);
        let pc = int.registers.program_counter;
        int.execute(Instruction::SNE(REG_1, 1)).unwrap();
        assert_eq!(int.registers.program_counter, pc + 2);

        let pc = int.registers.program_counter;
        int.execute(Instruction::SNE(REG_1, 0)).unwrap();
        assert_ne!(int.registers.program_counter, pc + 2);
    }

    #[test]
    fn instruction_se_reg() {
        let mut int = create_cpu();

        let pc = int.registers.program_counter;
        int.set_vreg(REG_1, 1);
        int.set_vreg(REG_2, 1);
        int.execute(Instruction::SE_Reg(REG_1, REG_2)).unwrap();
        assert_eq!(int.registers.program_counter, pc + 2);

        let pc = int.registers.program_counter;
        int.set_vreg(REG_2, 2);
        int.execute(Instruction::SE_Reg(REG_1, REG_2)).unwrap();
        assert_ne!(int.registers.program_counter, pc + 2);
    }

    #[test]
    fn timers_underflow() {
        let mut cpu = create_cpu();

        assert_eq!(cpu.registers.delay_timer, 0);
        assert_eq!(cpu.registers.sound_timer, 0);

        cpu.decrease_delay_timer();
        cpu.decrease_sound_timer();

        assert_eq!(cpu.registers.delay_timer, 0);
        assert_eq!(cpu.registers.sound_timer, 0);
    }
}
