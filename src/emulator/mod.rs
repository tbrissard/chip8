use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
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
    /// Asks the emulator to refresh the shared states
    RefreshSharedStates,
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

/// States buffer read by UI
#[derive(Debug, Default)]
pub(crate) struct Shared {
    pub(crate) screen: StandardScreen,
    pub(crate) keyboard: Ch8Keyboard,
    pub(crate) registers: Registers,
    pub(crate) stats: Stats,
}

/// Useful stats
#[derive(Debug, Clone)]
pub(crate) struct Stats {
    /// Time spent running
    pub(crate) uptime: Duration,
    pub(crate) last_restart: Instant,
    pub(crate) last_pause: Option<Instant>,
    /// Number of instructions executed
    pub(crate) cycles: u16,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            uptime: Duration::ZERO,
            last_restart: Instant::now(),
            last_pause: None,
            cycles: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Emulator {
    regs: Registers,
    memory: Memory,
    screen: StandardScreen,
    keyboard: Ch8Keyboard,

    pub(super) shared: Arc<Mutex<Shared>>,
    state: EmulatorState,
    cpu_mode: CpuMode,

    pub(super) cycle_interval: Duration,
    next_cycle: Instant,
    timer_tick_interval: Duration,
    next_timer_tick: Instant,

    /// Executed instructions
    pub(crate) history: Vec<Instruction>,
    stats: Stats,
}

const TIMER_TICK_RATE: f64 = 60.0;
const DEFAULT_CLOCK_SPEED: f64 = 60.0;
const START_ADDRESS: Address = 0x200;

impl Default for Emulator {
    fn default() -> Self {
        let registers = Registers {
            pc: START_ADDRESS,
            ..Default::default()
        };

        let now = Instant::now();

        Self {
            regs: registers.clone(),
            memory: Memory::default(),
            screen: StandardScreen::default(),
            keyboard: Ch8Keyboard::default(),

            shared: Arc::new(Mutex::new(Shared {
                screen: StandardScreen::default(),
                keyboard: Ch8Keyboard::default(),
                registers,
                stats: Stats::default(),
            })),
            state: EmulatorState::Running(RunMode::Standard),
            cpu_mode: CpuMode::default(),

            cycle_interval: Duration::from_secs_f64(1.0 / DEFAULT_CLOCK_SPEED),
            next_cycle: now,
            timer_tick_interval: Duration::from_secs_f64(1.0 / TIMER_TICK_RATE),
            next_timer_tick: now,

            history: Vec::new(),
            stats: Stats::default(),
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

    fn update_shared_states(&mut self) {
        let shared = &mut self.shared.lock().unwrap();
        shared.screen = self.screen.clone();
        shared.keyboard = self.keyboard.clone();
        shared.registers = self.regs.clone();
        shared.stats = self.stats.clone();
    }

    pub(super) fn run(&mut self, rx: Receiver<EmulatorMessage>) {
        self.stats.last_restart = Instant::now();

        while self.state != EmulatorState::Stopped {
            let messages = Self::poll_messages(&rx);
            for msg in messages {
                self.handle_message(msg);
            }

            if let EmulatorState::Running(run_mode) = self.state {
                if Instant::now() > self.next_timer_tick {
                    self.regs.delay_timer = self.regs.delay_timer.saturating_sub(1);
                    self.regs.sound_timer = self.regs.sound_timer.saturating_sub(1);
                    self.keyboard.release_keys();
                    self.next_timer_tick += self.timer_tick_interval;
                }

                if Instant::now() >= self.next_cycle && self.cpu_mode == CpuMode::Active {
                    let pc = self.regs.pc;
                    let last_instr = self.step().unwrap();
                    if self.regs.pc == pc {
                        // execution address has not changed, the program has entered a dead state
                        self.stop();
                        break;
                    }

                    self.history.push(last_instr);
                    self.stats.cycles += 1;
                    self.next_cycle += self.cycle_interval;

                    if run_mode == RunMode::Debug {
                        self.pause();
                        break;
                    }
                }

                thread::sleep(
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

            (EmulatorMessage::RefreshSharedStates, EmulatorState::Running(_)) => {
                self.update_shared_states()
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
        self.stats.last_pause = Some(now);
        self.stats.uptime += now.duration_since(self.stats.last_restart);
        self.state = EmulatorState::Paused;
    }

    /// Resume the emulator's execution
    fn resume(&mut self, run_mode: RunMode) {
        let now = Instant::now();
        let pause_duration = now.duration_since(
            self.stats
                .last_pause
                .take()
                .expect("emulator was not paused properly (missing last_pause)"),
        );
        self.next_cycle += pause_duration;
        self.next_timer_tick += pause_duration;
        self.stats.last_restart = now;
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
        let a = self.memory.read(self.regs.pc, 2)?;
        let a = <&[u8; 2]>::try_from(a).unwrap();
        let instr = std::convert::TryInto::<Instruction>::try_into(a)?;
        self.advance_pc();
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
            Instruction::CLS => self.screen.clear(),
            Instruction::RET => {
                let addr = self.regs.pop_stack()?;
                self.set_pc(addr);
            }
            Instruction::JP(addr) => self.set_pc(addr),
            Instruction::CALL(addr) => {
                self.regs.push_stack(self.regs.pc)?;
                self.set_pc(addr);
            }
            Instruction::SE_Value(vx, kk) => {
                if self.regs.vreg(vx) == kk {
                    self.advance_pc();
                }
            }
            Instruction::SNE(vx, kk) => {
                if self.regs.vreg(vx) != kk {
                    self.advance_pc();
                }
            }
            Instruction::SE_Reg(vx, vy) => {
                if self.vreg(vx) == self.vreg(vy) {
                    self.advance_pc();
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
                    self.advance_pc();
                }
            }
            Instruction::LD_I(addr) => self.regs.i = addr,
            Instruction::JP_V0(addr) => {
                self.set_pc(Address::from(self.vreg(VRegister::V0)) + addr);
            }
            Instruction::RND(vx, kk) => {
                let mut rng = rand::rng();
                let rnd: u8 = rng.random();
                self.set_vreg(vx, rnd & kk);
            }
            Instruction::DRW(vx, vy, n) => {
                let sprite = self.memory.read(self.regs.i, Address::from(n))?.into();
                let collision = self.screen.write_sprite(
                    &sprite,
                    self.vreg(vx) as usize,
                    self.vreg(vy) as usize,
                );
                self.set_f(collision.into());
            }
            Instruction::SKP(vx) => {
                if self.keyboard.is_down(self.vreg(vx).try_into()?) {
                    self.advance_pc();
                }
            }
            Instruction::SKNP(vx) => {
                if self.keyboard.is_up(self.vreg(vx).try_into()?) {
                    self.advance_pc();
                }
            }
            Instruction::LD_DT(vx) => self.set_vreg(vx, self.regs.delay_timer),
            Instruction::LD_K(vx) => self.cpu_mode = CpuMode::WaitingForKey(vx),
            Instruction::SET_DT(vx) => self.regs.delay_timer = self.vreg(vx),
            Instruction::SET_ST(vx) => self.regs.sound_timer = self.vreg(vx),
            Instruction::ADD_I(vx) => self.regs.i += Address::from(self.vreg(vx)),
            Instruction::LD_F(vx) => self.regs.i = memory::digit_addr(self.vreg(vx)),
            Instruction::LD_B(vx) => {
                let value = self.vreg(vx);
                self.memory.store(&[value / 100], self.regs.i)?;
                self.memory.store(&[value % 100 / 10], self.regs.i + 1)?;
                self.memory.store(&[value % 10], self.regs.i + 2)?;
            }
            Instruction::LD_MEM_I(vx) => {
                for (value, addr) in self
                    .regs
                    .values
                    .iter()
                    .take(vx as usize + 1)
                    .zip(self.regs.i..)
                {
                    self.memory.store(&[*value], addr)?;
                }
            }
            Instruction::LD_I_MEM(vx) => {
                for (reg, addr) in self
                    .regs
                    .values
                    .iter_mut()
                    .take(vx as usize + 1)
                    .zip(self.regs.i..)
                {
                    *reg = self.memory.read(addr, 1)?[0];
                }
            }
        }

        Ok(())
    }

    /// Advance the program counter to the next instruction
    fn advance_pc(&mut self) {
        self.regs.pc += 2;
    }

    /// Set the program counter to an arbitrary address
    fn set_pc(&mut self, addr: Address) {
        self.regs.pc = addr;
    }

    /// Set the VF register
    fn set_f(&mut self, value: ValueRegister) {
        self.set_vreg(VRegister::VF, value);
    }

    fn vreg(&self, vreg: VRegister) -> ValueRegister {
        self.regs.vreg(vreg)
    }

    pub(crate) fn set_vreg(&mut self, vreg: VRegister, value: ValueRegister) {
        self.regs.set_vreg(vreg, value);
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
        assert_eq!(int.regs.pc, ADDR);
    }

    #[test]
    fn instruction_call() {
        let mut int = create_cpu();

        let pc = int.regs.pc;

        int.execute(Instruction::CALL(ADDR)).unwrap();
        assert_eq!(pc, int.regs._top_stack().unwrap());
        assert_eq!(int.regs.pc, ADDR);
    }

    #[test]
    fn instruction_se_value() {
        let mut int = create_cpu();

        int.set_vreg(REG_1, 1);
        let pc = int.regs.pc;
        int.execute(Instruction::SE_Value(REG_1, 1)).unwrap();
        assert_eq!(int.regs.pc, pc + 2);

        let pc = int.regs.pc;
        int.execute(Instruction::SE_Value(REG_1, 2)).unwrap();
        assert_ne!(int.regs.pc, pc + 2);
    }

    #[test]
    fn instruction_sne() {
        let mut int = create_cpu();

        int.set_vreg(REG_1, 0);
        let pc = int.regs.pc;
        int.execute(Instruction::SNE(REG_1, 1)).unwrap();
        assert_eq!(int.regs.pc, pc + 2);

        let pc = int.regs.pc;
        int.execute(Instruction::SNE(REG_1, 0)).unwrap();
        assert_ne!(int.regs.pc, pc + 2);
    }

    #[test]
    fn instruction_se_reg() {
        let mut int = create_cpu();

        let pc = int.regs.pc;
        int.set_vreg(REG_1, 1);
        int.set_vreg(REG_2, 1);
        int.execute(Instruction::SE_Reg(REG_1, REG_2)).unwrap();
        assert_eq!(int.regs.pc, pc + 2);

        let pc = int.regs.pc;
        int.set_vreg(REG_2, 2);
        int.execute(Instruction::SE_Reg(REG_1, REG_2)).unwrap();
        assert_ne!(int.regs.pc, pc + 2);
    }
}
