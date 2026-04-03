use std::time::{Duration, Instant};

use rand::{RngExt, rngs::ThreadRng};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, KeyCode, KeyEvent},
    layout::{Constraint, Layout},
};

use crate::{
    cpu::{
        history::History,
        instruction::{Instruction, InstructionsError},
        registers::{Registers, RegistersError, VRegister},
    },
    keyboard::{Ch8Key, Ch8Keyboard, KeyboardError},
    memory::{self, Address, Memory, MemoryError},
    screen::StandardScreen,
};

mod history;
mod instruction;
mod registers;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Running,
    Paused(PauseOrigin),
    Terminated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PauseOrigin {
    UserPause,
    LoadKeyInstruction(VRegister),
}

#[derive(Debug)]
pub struct Cpu {
    registers: Registers,
    keyboard: Ch8Keyboard,
    memory: Memory,
    pub(crate) screen: StandardScreen,

    history: History,

    tick_interval: Duration,
    next_tick: Instant,
    frame_interval: Duration,
    next_frame: Instant,

    rng: ThreadRng,
    state: State,
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

            history: History::new(),

            tick_interval: Duration::from_secs_f64(1.0 / Self::DEFAULT_CLOCK_SPEED),
            next_tick: Instant::now(),
            frame_interval: Duration::from_secs_f64(1.0 / Self::FRAME_RATE),
            next_frame: Instant::now(),

            rng: ThreadRng::default(),
            state: State::Running,
        }
    }
}

impl Cpu {
    const DEFAULT_CLOCK_SPEED: f64 = 60.0;
    const FRAME_RATE: f64 = 60.0;

    pub(crate) fn load_program(bytes: &[u8]) -> Result<Cpu, ExecutionError> {
        let mut cpu = Self::default();
        cpu.memory.store(bytes, START_ADDRESS)?;
        Ok(cpu)
    }

    pub(crate) fn with_clock_speed(mut self, frequency: u64) -> Self {
        self.tick_interval = Duration::from_millis(1000 / frequency);
        self
    }

    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), ExecutionError> {
        while self.state != State::Terminated {
            self.poll_event()?;

            if self.state == State::Running {
                let pc = self.registers.program_counter;

                let instr = self.next_instr()?;
                self.execute(instr)?;

                if self.registers.program_counter == pc {
                    self.terminate();
                };

                if Instant::now() > self.next_frame {
                    self.keyboard.release_keys();
                    self.next_frame += self.frame_interval;
                    self.decrease_delay_timer();
                    self.decrease_sound_timer();
                    terminal
                        .draw(|frame| self.draw(frame))
                        .map_err(ExecutionError::Drawing)?;
                }
            }

            std::thread::sleep(self.next_tick.saturating_duration_since(Instant::now()));
            self.next_tick += self.tick_interval;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::horizontal(vec![
            Constraint::Length(StandardScreen::WIDTH as u16 + 2),
            Constraint::Length(17),
            Constraint::Length(35),
        ])
        .split(frame.area());

        let inner_layout = Layout::vertical(vec![
            Constraint::Length(StandardScreen::HEIGHT as u16 + 2),
            Constraint::Length(5),
        ])
        .split(layout[0]);

        frame.render_widget(&self.screen, inner_layout[0]);
        frame.render_widget(&self.keyboard, inner_layout[1]);
        frame.render_widget(&self.history, layout[1]);
        frame.render_widget(&self.registers, layout[2]);
    }

    fn pause(&mut self) {
        self.state = State::Paused(PauseOrigin::UserPause);
    }

    fn resume(&mut self) {
        self.state = State::Running;
        self.next_tick = Instant::now() + self.tick_interval;
        self.next_frame = Instant::now() + self.frame_interval;
    }

    fn terminate(&mut self) {
        self.state = State::Terminated;
    }

    fn poll_event(&mut self) -> Result<(), ExecutionError> {
        if event::poll(self.frame_interval / 10).map_err(ExecutionError::Event)?
            && let event::Event::Key(key_event) = event::read().map_err(ExecutionError::Event)?
        {
            self.handle_key_event(key_event);
        }

        Ok(())
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        match Ch8Key::try_from(event.code) {
            Ok(ch8_key) => {
                if let State::Paused(PauseOrigin::LoadKeyInstruction(vx)) = self.state {
                    self.set_v_reg(vx, ch8_key.into());
                    self.resume();
                } else {
                    self.keyboard.handle_ch8_key_event(ch8_key, event.kind)
                }
            }

            Err(KeyboardError::KeyNotBound(key_code)) => match key_code {
                KeyCode::Char('q') => self.terminate(),
                KeyCode::Char('p') => match self.state {
                    State::Paused(PauseOrigin::UserPause) => self.resume(),
                    State::Running => self.pause(),
                    _ => {}
                },
                _ => {}
            },

            Err(KeyboardError::InvalidKeyValue(_)) => panic!("should not happen"),
        }
    }

    fn next_instr(&mut self) -> Result<Instruction, ExecutionError> {
        let a = self.memory.read(self.registers.program_counter, 2)?;
        let a = <&[u8; 2]>::try_from(a).unwrap();
        let instr: Instruction = a.try_into()?;
        self.registers.program_counter += 2;
        Ok(instr)
    }

    fn execute(&mut self, instr: Instruction) -> Result<(), ExecutionError> {
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
                if self.v_reg(vx) == kk {
                    self.skip_instr();
                }
            }

            Instruction::SNE(vx, kk) => {
                if self.v_reg(vx) != kk {
                    self.skip_instr();
                }
            }

            Instruction::SE_Reg(vx, vy) => {
                if self.v_reg(vx) == self.v_reg(vy) {
                    self.skip_instr();
                }
            }

            Instruction::LD(vx, kk) => self.set_v_reg(vx, kk),

            Instruction::ADD(vx, kk) => self.set_v_reg(vx, self.v_reg(vx).wrapping_add(kk)),

            Instruction::LD_Regs(vx, vy) => self.set_v_reg(vx, self.v_reg(vy)),

            Instruction::OR(vx, vy) => self.set_v_reg(vx, self.v_reg(vx) | self.v_reg(vy)),

            Instruction::AND(vx, vy) => self.set_v_reg(vx, self.v_reg(vx) & self.v_reg(vy)),

            Instruction::XOR(vx, vy) => self.set_v_reg(vx, self.v_reg(vx) ^ self.v_reg(vy)),

            Instruction::ADD_Reg(vx, vy) => {
                let (res, carry) = self.v_reg(vx).overflowing_add(self.v_reg(vy));
                self.set_v_reg(vx, res);
                self.set_f(carry.into());
            }

            Instruction::SUB(vx, vy) => {
                let (res, carry) = self.v_reg(vx).overflowing_sub(self.v_reg(vy));
                self.set_v_reg(vx, res);
                self.set_f((!carry).into());
            }

            Instruction::SHR(vx) => {
                let value = self.v_reg(vx);
                self.set_f(value & 1);
                self.set_v_reg(vx, value >> 1);
            }

            Instruction::SUBN(vx, vy) => {
                let (res, carry) = self.v_reg(vy).overflowing_sub(self.v_reg(vx));
                self.set_v_reg(vx, res);
                self.set_f((!carry).into());
            }

            Instruction::SHL(vx) => {
                let value = self.v_reg(vx);
                self.set_f(value & 1);
                self.set_v_reg(vx, value << 1);
            }

            Instruction::SNE_Reg(vx, vy) => {
                if self.v_reg(vx) != self.v_reg(vy) {
                    self.skip_instr();
                }
            }

            Instruction::LD_I(addr) => self.registers.i = addr,

            Instruction::JP_V0(addr) => {
                self.set_pc(self.v_reg(0) as Address + addr);
            }

            Instruction::RND(vx, kk) => {
                let rnd: u8 = self.rng.random();
                self.set_v_reg(vx, rnd & kk);
            }

            Instruction::DRW(vx, vy, n) => {
                let sprite = self.memory.read(self.registers.i, n as Address)?.into();
                let collision = self.screen.write_sprite(
                    &sprite,
                    self.v_reg(vx) as usize,
                    self.v_reg(vy) as usize,
                );
                self.set_f(collision.into());
            }

            Instruction::SKP(vx) => {
                if self.keyboard.is_down(self.v_reg(vx).try_into()?) {
                    self.skip_instr();
                }
            }

            Instruction::SKNP(vx) => {
                if self.keyboard.is_up(self.v_reg(vx).try_into()?) {
                    self.skip_instr();
                }
            }

            Instruction::LD_DT(vx) => self.set_v_reg(vx, self.registers.delay_timer),

            Instruction::LD_K(vx) => {
                self.state = State::Paused(PauseOrigin::LoadKeyInstruction(vx))
            }

            Instruction::SET_DT(vx) => self.registers.delay_timer = self.v_reg(vx),

            Instruction::SET_ST(vx) => self.registers.sound_timer = self.v_reg(vx),

            Instruction::ADD_I(vx) => self.registers.i += self.v_reg(vx) as Address,

            Instruction::LD_F(vx) => self.registers.i = memory::digit_addr(self.v_reg(vx)),

            Instruction::LD_B(vx) => {
                let value = self.v_reg(vx);
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

        self.history.push(instr);

        Ok(())
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

    fn decrease_delay_timer(&mut self) {
        self.registers.delay_timer = self.registers.delay_timer.saturating_sub(1);
    }

    fn decrease_sound_timer(&mut self) {
        self.registers.sound_timer = self.registers.sound_timer.saturating_sub(1);
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

    #[error("event error: {0}")]
    Event(std::io::Error),
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

        int.registers.v_registers[0] = 1;
        let pc = int.registers.program_counter;
        int.execute(Instruction::SE_Value(0, 1)).unwrap();
        assert_eq!(int.registers.program_counter, pc + 2);

        let pc = int.registers.program_counter;
        int.execute(Instruction::SE_Value(0, 2)).unwrap();
        assert_ne!(int.registers.program_counter, pc + 2);
    }

    #[test]
    fn instruction_sne() {
        let mut int = create_cpu();

        int.registers.v_registers[0] = 0;
        let pc = int.registers.program_counter;
        int.execute(Instruction::SNE(0, 1)).unwrap();
        assert_eq!(int.registers.program_counter, pc + 2);

        let pc = int.registers.program_counter;
        int.execute(Instruction::SNE(0, 0)).unwrap();
        assert_ne!(int.registers.program_counter, pc + 2);
    }

    #[test]
    fn instruction_se_reg() {
        let mut int = create_cpu();

        let pc = int.registers.program_counter;
        int.registers.v_registers[0] = 1;
        int.registers.v_registers[1] = 1;
        int.execute(Instruction::SE_Reg(0, 1)).unwrap();
        assert_eq!(int.registers.program_counter, pc + 2);

        let pc = int.registers.program_counter;
        int.registers.v_registers[1] = 0;
        int.execute(Instruction::SE_Reg(0, 1)).unwrap();
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
