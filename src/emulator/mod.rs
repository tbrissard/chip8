use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rand::RngExt;

pub(crate) use crate::emulator::instruction::{Instruction, InstructionError};
pub(crate) use crate::emulator::registers::Registers;
pub(crate) use crate::emulator::registers::VRegister;
pub(crate) use crate::emulator::registers::ValueRegister;
use crate::keyboard::Ch8Key;
pub(crate) use crate::memory::MemoryError;
use crate::{
    emulator::registers::RegistersError,
    keyboard::{Ch8Keyboard, KeyError},
    memory::{self, Address, Memory},
    screen::StandardScreen,
};

mod instruction;
mod registers;

#[derive(Debug, Clone, Copy)]
pub(crate) enum EmulatorMessage {
    /// Pause/Resume the emulator
    TogglePause,
    /// Only execute the next instruction, then pause the emulator
    Step,
    /// Stop the emulator
    Stop,
    /// Press a key on the virtual keyboard
    KeyPress(Ch8Key),
    // /// Change the clock speed
    // _ChangeClockSpeed(f64),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    #[default]
    Standard,
    /// The emulator will enter the [Paused] state after the next instruction
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmulatorState {
    /// The emulator runs
    Running(RunMode),
    /// The execution is paused
    Paused,
    /// The emulator has reached the end of execution
    Stopped,
}

impl Default for EmulatorState {
    fn default() -> Self {
        Self::Running(RunMode::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum CpuMode {
    #[default]
    Active,
    /// The cpu is waiting for a key press (see instruction `LD_K`)
    WaitingForKey(VRegister),
}

/// Holds shared states needed by UI for granular locking
#[derive(Debug, Default)]
pub(crate) struct Shared {
    pub(crate) screen: StandardScreen,
}

#[derive(Debug, Clone)]
pub struct Emulator {
    pub(crate) registers: Registers,
    pub(crate) keyboard: Ch8Keyboard,
    memory: Memory,

    pub(super) shared: Arc<Mutex<Shared>>,
    state: EmulatorState,
    cpu_mode: CpuMode,

    pub(super) cycle_interval: Duration,
    next_cycle: Instant,
    timer_tick_interval: Duration,
    next_timer_tick: Instant,

    /// Executed instructions
    pub(crate) history: Vec<Instruction>,
    // self.emulator_uptime += Instant::now().saturating_duration_since(emulator_start);
    /// Time spent running
    pub(crate) uptime: Duration,
    last_restart: Instant,
    last_pause: Option<Instant>,
    /// Number of instructions executed
    pub(crate) cycles: u16,
}

const TIMER_TICK_RATE: f64 = 60.0;
const DEFAULT_CLOCK_SPEED: f64 = 60.0;
const START_ADDRESS: Address = 0x200;

impl Default for Emulator {
    fn default() -> Self {
        let registers = Registers {
            program_counter: START_ADDRESS,
            ..Default::default()
        };

        let now = Instant::now();

        Self {
            registers,
            keyboard: Ch8Keyboard::new(),
            memory: Memory::default(),

            shared: Arc::new(Mutex::new(Shared {
                screen: StandardScreen::default(),
            })),
            state: EmulatorState::Running(RunMode::Standard),
            cpu_mode: CpuMode::default(),

            cycle_interval: Duration::from_secs_f64(1.0 / DEFAULT_CLOCK_SPEED),
            next_cycle: now,
            timer_tick_interval: Duration::from_secs_f64(1.0 / TIMER_TICK_RATE),
            next_timer_tick: now,

            history: Vec::new(),
            uptime: Duration::ZERO,
            last_restart: now,
            last_pause: None,
            cycles: 0,
        }
    }
}

impl Emulator {
    pub(crate) fn load_program(bytes: &[u8]) -> Result<Self, MemoryError> {
        let mut cpu = Self::default();
        cpu.memory.store(bytes, START_ADDRESS)?;
        Ok(cpu)
    }

    fn poll_messages(rx: &Receiver<EmulatorMessage>) -> Vec<EmulatorMessage> {
        let mut messages = Vec::new();
        loop {
            match rx.try_recv() {
                Ok(msg) => messages.push(msg),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    panic!("cannot receive messages, channel disconnected")
                }
            }
        }
        messages
    }

    pub(super) fn run(&mut self, rx: Receiver<EmulatorMessage>) {
        self.last_restart = Instant::now();

        while self.state != EmulatorState::Stopped {
            let messages = Self::poll_messages(&rx);
            for msg in messages {
                self.handle_message(msg);
            }

            if let EmulatorState::Running(run_mode) = self.state {
                if Instant::now() > self.next_timer_tick {
                    self.decrement_timers();
                    self.keyboard.release_keys();
                    self.next_timer_tick += self.timer_tick_interval;
                }

                if Instant::now() >= self.next_cycle && self.cpu_mode == CpuMode::Active {
                    let pc = self.registers.program_counter;
                    let last_instr = self.step().unwrap();
                    if self.registers.program_counter == pc {
                        // execution address has not changed, the program has entered a dead state
                        self.stop();
                        break;
                    }

                    self.history.push(last_instr);
                    self.cycles += 1;
                    self.next_cycle += self.cycle_interval;

                    if run_mode == RunMode::Debug {
                        self.pause();
                        break;
                    }
                }

                std::thread::sleep(
                    self.next_cycle
                        .min(self.next_timer_tick)
                        .saturating_duration_since(Instant::now()),
                );
            }
        }
    }

    fn handle_message(&mut self, message: EmulatorMessage) {
        match (message, self.state) {
            (EmulatorMessage::TogglePause, EmulatorState::Running(_)) => self.pause(),
            (EmulatorMessage::TogglePause, EmulatorState::Paused) => self.resume(RunMode::Standard),

            (EmulatorMessage::Step, EmulatorState::Paused) => self.resume(RunMode::Debug),
            (EmulatorMessage::Step, EmulatorState::Running(RunMode::Standard)) => self.pause(),

            (EmulatorMessage::Stop, _) => self.stop(),

            (EmulatorMessage::KeyPress(ch8_key), EmulatorState::Running(_)) => {
                self.press_key(ch8_key)
            }

            (_, _) => {}
        }
    }

    fn stop(&mut self) {
        self.state = EmulatorState::Stopped;
    }

    /// Pause the emulator's execution
    fn pause(&mut self) {
        let now = Instant::now();
        self.last_pause = Some(now);
        self.uptime += now.duration_since(self.last_restart);
        self.state = EmulatorState::Paused;
    }

    /// Resume the emulator's execution
    fn resume(&mut self, run_mode: RunMode) {
        let now = Instant::now();
        let pause_duration = now.duration_since(
            self.last_pause
                .take()
                .expect("emulator was not paused properly (missing last_pause)"),
        );
        self.next_cycle += pause_duration;
        self.next_timer_tick += pause_duration;
        self.last_restart = now;
        self.state = EmulatorState::Running(run_mode);
    }

    fn press_key(&mut self, key: Ch8Key) {
        self.keyboard.press_key(key);
        if let CpuMode::WaitingForKey(vx) = self.cpu_mode {
            self.set_vreg(vx, Into::<u8>::into(key));
            self.cpu_mode = CpuMode::Active;
        }
    }

    /// Fetch the next instruction
    fn next_instr(&mut self) -> Result<Instruction, InstructionFetchError> {
        let a = self.memory.read(self.registers.program_counter, 2)?;
        let a = <&[u8; 2]>::try_from(a).unwrap();
        let instr = std::convert::TryInto::<Instruction>::try_into(a)?;
        self.registers.program_counter += 2;
        Ok(instr)
    }

    /// Execute the next instruction and returns it
    pub(super) fn step(&mut self) -> Result<Instruction, StepError> {
        let instr = self.next_instr()?;
        self.execute(instr)
            .map_err(|e| StepError::Execution(instr, e))?;
        Ok(instr)
    }

    /// Execute an instruction
    fn execute(&mut self, instr: Instruction) -> Result<(), ExecutionError> {
        match instr {
            Instruction::CLS => self.shared.lock().unwrap().screen.clear(),
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
                self.set_pc(Address::from(self.vreg(VRegister::V0)) + addr);
            }
            Instruction::RND(vx, kk) => {
                let mut rng = rand::rng();
                let rnd: u8 = rng.random();
                self.set_vreg(vx, rnd & kk);
            }
            Instruction::DRW(vx, vy, n) => {
                let sprite = self.memory.read(self.registers.i, Address::from(n))?.into();
                let collision = self.shared.lock().unwrap().screen.write_sprite(
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
            Instruction::LD_K(vx) => self.cpu_mode = CpuMode::WaitingForKey(vx),
            Instruction::SET_DT(vx) => self.registers.delay_timer = self.vreg(vx),
            Instruction::SET_ST(vx) => self.registers.sound_timer = self.vreg(vx),
            Instruction::ADD_I(vx) => self.registers.i += Address::from(self.vreg(vx)),
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
                    .values
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
                    .values
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

    fn skip_instr(&mut self) {
        self.registers.program_counter += 2;
    }

    fn vreg(&self, vreg: VRegister) -> ValueRegister {
        self.registers.vreg(vreg)
    }

    pub(crate) fn set_vreg(&mut self, vreg: VRegister, value: ValueRegister) {
        self.registers.set_vreg(vreg, value);
    }

    fn set_f(&mut self, value: ValueRegister) {
        self.set_vreg(VRegister::VF, value);
    }

    fn set_pc(&mut self, addr: Address) {
        self.registers.program_counter = addr;
    }

    pub(crate) fn decrement_timers(&mut self) {
        self.registers.delay_timer = self.registers.delay_timer.saturating_sub(1);
        self.registers.sound_timer = self.registers.sound_timer.saturating_sub(1);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StepError {
    #[error("could not fetch instruction: {0}")]
    InstructionFetch(#[from] InstructionFetchError),

    #[error("could not execute {0}: {1}")]
    Execution(Instruction, ExecutionError),
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

    fn create_cpu() -> Emulator {
        Emulator::default()
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

        cpu.decrement_timers();

        assert_eq!(cpu.registers.delay_timer, 0);
        assert_eq!(cpu.registers.sound_timer, 0);
    }
}
