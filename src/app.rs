use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use ratatui::DefaultTerminal;

use crate::{
    cpu::{
        Cpu, ExecutionError, ExecutionResult, Instruction, InstructionFetchError, MemoryError,
        Registers, VRegister,
    },
    input,
    keyboard::{Ch8Key, Ch8Keyboard},
    screen::StandardScreen,
    tui,
};

pub type Result<T> = std::result::Result<T, Chip8Error>;

const DEFAULT_CLOCK_SPEED: f64 = 60.0;
const FRAME_RATE: f64 = 60.0;

#[derive(Debug, Clone)]
pub(crate) enum Action {
    Quit,
    TogglePause,
    Chip8KeyPress(Ch8Key),
    _LoadProgram(Vec<u8>),
    _ChangeClockSpeed(f64),
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

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
pub(crate) struct App {
    history: Vec<Instruction>,
    pub(crate) cpu: Cpu,

    state: State,

    tick_interval: Duration,
    next_tick: Instant,
    frame_interval: Duration,
    next_frame: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            cpu: Cpu::default(),

            state: State::Running,

            tick_interval: Duration::from_secs_f64(1.0 / DEFAULT_CLOCK_SPEED),
            next_tick: Instant::now(),
            frame_interval: Duration::from_secs_f64(1.0 / FRAME_RATE),
            next_frame: Instant::now(),
        }
    }
}

impl App {
    pub fn set_clock_speed(&mut self, frequency: f64) {
        self.tick_interval = Duration::from_secs_f64(1.0 / frequency)
    }

    fn terminate(&mut self) {
        self.state = State::Terminated;
    }

    fn pause(&mut self) {
        self.state = State::Paused(PauseOrigin::UserPause);
    }

    fn resume(&mut self) {
        self.state = State::Running;
        self.next_tick = Instant::now() + self.tick_interval;
        self.next_frame = Instant::now() + self.frame_interval;
    }

    pub fn load_program(&mut self, bytes: &[u8]) -> Result<()> {
        self.cpu = Cpu::load_program(bytes).map_err(Chip8Error::ProgramLoadingFailed)?;
        self.history.clear();
        Ok(())
    }

    pub(crate) fn handle_action(&mut self, action: &Action) -> Result<()> {
        match action {
            Action::Quit => self.terminate(),

            Action::TogglePause => match self.state {
                State::Running => self.pause(),
                State::Paused(PauseOrigin::UserPause) => self.resume(),
                _ => {}
            },

            Action::Chip8KeyPress(ch8_key) => {
                if let State::Paused(PauseOrigin::LoadKeyInstruction(vx)) = self.state {
                    self.cpu.set_v_reg(vx, Into::<u8>::into(*ch8_key));
                    self.resume();
                } else {
                    self.cpu.keyboard.press_key(*ch8_key)
                }
            }

            Action::_LoadProgram(bytes) => {
                self.load_program(bytes)?;
            }

            Action::_ChangeClockSpeed(frequency) => self.set_clock_speed(*frequency),
        }
        Ok(())
    }

    pub(crate) fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> std::result::Result<(), RunError> {
        while self.state != State::Terminated {
            for a in input::poll_action().map_err(RunError::ActionPollFailed)? {
                self.handle_action(&a).map_err(|e| RunError::Action(a, e))?;
            }

            if self.state == State::Running {
                let pc = self.cpu.registers.program_counter;

                let instr = self.cpu.next_instr()?;
                let res = self
                    .cpu
                    .execute(instr)
                    .map_err(|e| RunError::Execution(instr, e))?;
                self.history.push(instr);
                match res {
                    ExecutionResult::Continue => {}
                    ExecutionResult::WaitForKey(vx) => {
                        self.state = State::Paused(PauseOrigin::LoadKeyInstruction(vx))
                    }
                }

                if self.cpu.registers.program_counter == pc {
                    self.terminate();
                };

                if Instant::now() > self.next_frame {
                    self.cpu.decrease_delay_timer();
                    self.cpu.decrease_sound_timer();
                    terminal
                        .draw(|frame| tui::draw(self, frame))
                        .map_err(RunError::RenderFailed)?;
                    self.cpu.keyboard.release_keys();
                    self.next_frame += self.frame_interval;
                }
            }

            std::thread::sleep(self.next_tick.saturating_duration_since(Instant::now()));
            self.next_tick += self.tick_interval;
        }

        Ok(())
    }

    pub(crate) fn registers(&self) -> &Registers {
        &self.cpu.registers
    }

    pub(crate) fn history(&self) -> &[Instruction] {
        &self.history
    }

    pub(crate) fn screen(&self) -> &StandardScreen {
        &self.cpu.screen
    }

    pub(crate) fn keyboard(&self) -> &Ch8Keyboard {
        &self.cpu.keyboard
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Chip8Error {
    #[error("could not load program: {0}")]
    ProgramLoadingFailed(MemoryError),
}

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("could not handle {0}: {1}")]
    Action(Action, Chip8Error),

    #[error("could not execute {0}: {1}")]
    Execution(Instruction, ExecutionError),

    #[error("could not fetch instruction: ")]
    InstructionFetch(#[from] InstructionFetchError),

    #[error("could not poll actions: {0}")]
    ActionPollFailed(std::io::Error),

    #[error("render failed: {0}")]
    RenderFailed(std::io::Error),
}
