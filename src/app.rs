use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use ratatui::DefaultTerminal;

use crate::input::EventReadingError;
use crate::{
    emulator::{CpuState, Emulator, Instruction, MemoryError, Registers, StepError},
    input::{self},
    keyboard::{Ch8Key, Ch8Keyboard},
    screen::StandardScreen,
    tui,
};

const DEFAULT_CLOCK_SPEED: f64 = 60.0;
const FRAME_RATE: f64 = 60.0;
const TIMER_TICK_RATE: f64 = 60.0;

#[derive(Debug, Clone)]
pub(crate) enum Action {
    /// Exit the emulator
    Quit,
    /// Resets the loaded program
    Reset,
    /// Pause/Resume the emulator
    TogglePause,
    /// Only execute the next instruction, then pause the emulator
    Step,
    /// The user pressed a key on the emulator keyboard
    Chip8KeyPress(Ch8Key),
    /// Change the clock speed
    _ChangeClockSpeed(f64),
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    #[default]
    Standard,
    /// The emulator will enter the [Paused] state after the next instruction
    Step,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmulatorState {
    Running(RunMode),
    /// The execution is paused
    Paused,
    Terminated,
}

impl Default for EmulatorState {
    fn default() -> Self {
        Self::Running(RunMode::default())
    }
}

#[derive(Debug)]
pub(crate) struct App {
    history: Vec<Instruction>,
    pub(crate) emulator: Emulator,

    emulator_state: EmulatorState,
    initial_snapshot: Emulator,

    emulator_tick_interval: Duration,
    next_emulator_tick: Instant,
    render_interval: Duration,
    next_render: Instant,
    emulator_frame_interval: Duration,
    next_emulator_frame: Instant,

    emulator_uptime: Duration,
    cycles_count: u32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            emulator: Emulator::default(),

            emulator_state: EmulatorState::default(),
            initial_snapshot: Emulator::default(),

            emulator_tick_interval: Duration::from_secs_f64(1.0 / DEFAULT_CLOCK_SPEED),
            next_emulator_tick: Instant::now(),
            render_interval: Duration::from_secs_f64(1.0 / FRAME_RATE),
            next_render: Instant::now(),
            emulator_frame_interval: Duration::from_secs_f64(1.0 / TIMER_TICK_RATE),
            next_emulator_frame: Instant::now(),

            emulator_uptime: Duration::ZERO,
            cycles_count: 0,
        }
    }
}

impl App {
    pub fn set_clock_speed(&mut self, frequency: f64) {
        self.emulator_tick_interval = Duration::from_secs_f64(1.0 / frequency)
    }

    fn terminate(&mut self) {
        self.emulator_state = EmulatorState::Terminated
    }

    fn reset(&mut self) {
        self.set_emulator_state(self.initial_snapshot.clone());
    }

    fn pause(&mut self) {
        self.emulator_state = EmulatorState::Paused
    }

    fn resume(&mut self) {
        self.emulator_state = EmulatorState::Running(RunMode::Standard);
        self.next_emulator_tick = Instant::now() + self.emulator_tick_interval;
        self.next_render = Instant::now() + self.render_interval;
        self.next_emulator_frame = Instant::now() + self.emulator_frame_interval;
    }

    fn step(&mut self) {
        self.emulator_state = EmulatorState::Running(RunMode::Step);
    }

    fn set_emulator_state(&mut self, emulator: Emulator) {
        self.emulator = emulator;
        self.cycles_count = 0;
        self.emulator_uptime = Duration::ZERO;
        self.history.clear();
    }

    pub fn load_rom(&mut self, bytes: &[u8]) -> Result<()> {
        let emu = Emulator::load_program(bytes).map_err(Chip8Error::ProgramLoadingFailed)?;
        self.set_emulator_state(emu);
        self.initial_snapshot = self.emulator.clone();
        Ok(())
    }

    pub(crate) fn handle_action(&mut self, action: &Action) {
        match (action, self.emulator_state) {
            (Action::Quit, _) => self.terminate(),

            (Action::Reset, EmulatorState::Running(_) | EmulatorState::Paused) => self.reset(),
            (Action::Reset, EmulatorState::Terminated) => {}

            (Action::TogglePause, EmulatorState::Running(_)) => self.pause(),
            (Action::TogglePause, EmulatorState::Paused) => self.resume(),
            (Action::TogglePause, _) => {}

            (Action::Step, EmulatorState::Running(RunMode::Standard) | EmulatorState::Paused) => {
                self.step()
            }
            (Action::Step, _) => {}

            (Action::Chip8KeyPress(ch8_key), EmulatorState::Running(_)) => {
                self.emulator.press_key(*ch8_key)
            }
            (Action::Chip8KeyPress(_), _) => {}

            (Action::_ChangeClockSpeed(frequency), _) => self.set_clock_speed(*frequency),
        }
    }

    /// Blocks until next event is available and process it
    fn process_next_event(&mut self) -> RunResult<()> {
        Action::from_event(input::read_event()?).inspect(|a| self.handle_action(a));
        Ok(())
    }

    /// Process available events
    fn process_events(&mut self) -> RunResult<()> {
        for action in input::poll_events()?
            .into_iter()
            .filter_map(Action::from_event)
        {
            self.handle_action(&action)
        }
        Ok(())
    }

    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> RunResult<()> {
        loop {
            self.process_events()?;

            match self.emulator_state {
                EmulatorState::Terminated => break,
                EmulatorState::Running(run_mode) => {
                    self.run_emulator(self.next_render, run_mode)?;
                    if run_mode == RunMode::Step {
                        self.pause();
                    }
                }
                EmulatorState::Paused => self.process_next_event()?,
            }

            if Instant::now() > self.next_render {
                terminal
                    .draw(|frame| tui::draw(self, frame))
                    .map_err(RunError::RenderFailed)?;
                self.next_render += self.render_interval;
            }
        }

        Ok(())
    }

    fn run_emulator(&mut self, until: Instant, mode: RunMode) -> RunResult<()> {
        let emulator_start = Instant::now();

        loop {
            if Instant::now() > self.next_emulator_tick
                && self.emulator.cpu_state == CpuState::Executing
            {
                let pc = self.emulator.registers.program_counter;
                let last_instr = self.emulator.step()?;
                if self.emulator.registers.program_counter == pc {
                    // execution address has not changed, the program has entered a dead state
                    self.terminate();
                    break;
                };

                self.history.push(last_instr);
                self.cycles_count += 1;
                self.next_emulator_tick += self.emulator_tick_interval;
            }

            if Instant::now() > self.next_emulator_frame {
                self.emulator.decrement_timers();
                self.emulator.keyboard.release_keys();
                self.next_emulator_frame += self.emulator_frame_interval;
            }

            if mode == RunMode::Step || Instant::now() > until {
                break;
            }

            std::thread::sleep(
                self.next_emulator_tick
                    .min(self.next_emulator_frame)
                    .min(until)
                    .saturating_duration_since(Instant::now()),
            );
        }

        self.emulator_uptime += Instant::now().saturating_duration_since(emulator_start);
        Ok(())
    }

    pub(crate) fn registers(&self) -> &Registers {
        &self.emulator.registers
    }

    pub(crate) fn history(&self) -> &[Instruction] {
        &self.history
    }

    pub(crate) fn screen(&self) -> &StandardScreen {
        &self.emulator.screen
    }

    pub(crate) fn keyboard(&self) -> &Ch8Keyboard {
        &self.emulator.keyboard
    }
}

pub type Result<T> = std::result::Result<T, Chip8Error>;

#[derive(Debug, thiserror::Error)]
pub enum Chip8Error {
    #[error("could not load program: {0}")]
    ProgramLoadingFailed(MemoryError),
}

type RunResult<T> = std::result::Result<T, RunError>;

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    // #[error("could not handle {0}: {1}")]
    // Action(Action, Chip8Error),
    #[error("emulator tick failed: {0}")]
    Execution(#[from] StepError),

    #[error("could not get events: {0}")]
    CouldNotGetEvents(#[from] EventReadingError),

    #[error("render failed: {0}")]
    RenderFailed(std::io::Error),
}
