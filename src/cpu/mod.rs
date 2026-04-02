use std::time::{Duration, Instant};

use rand::{RngExt, rngs::ThreadRng};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
};

use crate::{
    cpu::{
        history::History,
        instruction::{Instructions, InstructionsError},
        registers::{Registers, RegistersError, VRegister},
    },
    keyboard::{Keyboard, KeyboardError},
    memory::{self, Address, Memory, MemoryError},
    screen::StandardScreen,
};

mod history;
mod instruction;
mod registers;

#[derive(Debug)]
pub struct Cpu {
    registers: Registers,
    keyboard: Keyboard,
    memory: Memory,
    pub(crate) screen: StandardScreen,

    history: History,

    rng: ThreadRng,
    exit: bool,
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
            keyboard: Keyboard::default(),
            memory: Memory::default(),
            screen: StandardScreen::new(),

            history: History::new(),

            rng: ThreadRng::default(),
            exit: false,
        }
    }
}

impl Cpu {
    pub(crate) fn load_program(bytes: &[u8]) -> Result<Cpu, ExecutionError> {
        let mut cpu = Self::default();
        cpu.memory.store(bytes, START_ADDRESS)?;
        Ok(cpu)
    }

    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), ExecutionError> {
        const REFRESH_RATE: u64 = 60;
        let interval = Duration::from_millis(1000 / REFRESH_RATE);
        let mut next_refresh = Instant::now() + interval;

        while !self.exit {
            let pc = self.registers.program_counter;

            let instr = self.next_instr()?;
            self.execute(instr)?;

            if self.registers.program_counter == pc {
                self.exit = true;
            }

            terminal
                .draw(|frame| self.draw(frame))
                .map_err(ExecutionError::Drawing)?;

            self.decrease_delay_timer();
            self.decrease_sound_timer();

            std::thread::sleep(next_refresh - Instant::now());
            next_refresh += interval;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::horizontal(vec![
            Constraint::Length(StandardScreen::WIDTH as u16 + 2),
            Constraint::Length(17),
            Constraint::Length(35),
        ])
        // .flex(ratatui::layout::Flex::Start)
        .split(frame.area());

        let inner_layout = Layout::vertical(vec![
            Constraint::Length(StandardScreen::HEIGHT as u16 + 2),
            Constraint::Fill(1),
        ])
        .split(layout[0]);

        frame.render_widget(&self.screen, inner_layout[0]);
        frame.render_widget(&self.history, layout[1]);
        frame.render_widget(&self.registers, layout[2]);
    }

    fn skip_instr(&mut self) {
        self.registers.program_counter += 2;
    }

    fn next_instr(&mut self) -> Result<Instructions, ExecutionError> {
        let a = self.memory.read(self.registers.program_counter, 2)?;
        let a = <&[u8; 2]>::try_from(a).unwrap();
        let instr: Instructions = a.try_into()?;
        self.registers.program_counter += 2;
        Ok(instr)
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

    fn decrease_delay_timer(&mut self) {
        self.registers.delay_timer = self.registers.delay_timer.saturating_sub(1);
    }

    fn decrease_sound_timer(&mut self) {
        self.registers.sound_timer = self.registers.sound_timer.saturating_sub(1);
    }

    fn execute(&mut self, instr: Instructions) -> Result<(), ExecutionError> {
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

            // Instructions::ADD(vx, kk) => {self.set_v_reg(vx, self.v_reg(vx) + kk)},
            Instructions::ADD(vx, kk) => self.set_v_reg(vx, self.v_reg(vx).wrapping_add(kk)),

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

        self.history.push(instr);

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("register error: {0}")]
    Registers(#[from] RegistersError),

    #[error("memory error: {0}")]
    Memory(#[from] MemoryError),

    #[error("keyboard error: {0}")]
    Keyboard(#[from] KeyboardError),

    #[error("instruction error: {0}")]
    Instruction(#[from] InstructionsError),

    #[error("drawing error: {0}")]
    Drawing(std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADDR: Address = 0x321;

    fn create_cpu() -> Cpu {
        Cpu::default()
    }

    #[test]
    fn instruction_jp() {
        let mut int = create_cpu();

        let res = int.execute(Instructions::JP(ADDR));
        assert!(res.is_ok());
        assert_eq!(int.registers.program_counter, ADDR);
    }

    #[test]
    fn instruction_call() {
        let mut int = create_cpu();

        let pc = int.registers.program_counter;

        int.execute(Instructions::CALL(ADDR)).unwrap();
        assert_eq!(pc, int.registers._top_stack().unwrap());
        assert_eq!(int.registers.program_counter, ADDR);
    }

    #[test]
    fn instruction_se_value() {
        let mut int = create_cpu();

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
        let mut int = create_cpu();

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
        let mut int = create_cpu();

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
