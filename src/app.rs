use std::{
    marker::PhantomData,
    time::{Duration, Instant},
};

use ratatui::DefaultTerminal;

use crate::{
    emulator::{CpuState, Emulator, Instruction, MemoryError, Registers, StepError},
    input::InputManager,
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    #[default]
    Standard,
    /// The emulator will enter the [Paused] state after the next instruction
    Step,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EmulatorState {
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
pub(crate) struct App<T> {
    history: Vec<Instruction>,
    pub(crate) emulator: Emulator,
    input_manager: PhantomData<T>,

    pub(crate) emulator_state: EmulatorState,
    initial_snapshot: Emulator,

    emulator_cycle_interval: Duration,
    next_emulator_cycle: Instant,
    render_interval: Duration,
    next_render: Instant,
    emulator_frame_interval: Duration,
    next_emulator_frame: Instant,
}

impl<T> Default for App<T> {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            emulator: Emulator::default(),
            input_manager: PhantomData,

            emulator_state: EmulatorState::default(),
            initial_snapshot: Emulator::default(),

            emulator_cycle_interval: Duration::from_secs_f64(1.0 / DEFAULT_CLOCK_SPEED),
            next_emulator_cycle: Instant::now(),
            render_interval: Duration::from_secs_f64(1.0 / FRAME_RATE),
            next_render: Instant::now(),
            emulator_frame_interval: Duration::from_secs_f64(1.0 / TIMER_TICK_RATE),
            next_emulator_frame: Instant::now(),
        }
    }
}

impl<T: InputManager> App<T> {
    pub fn set_clock_speed(&mut self, frequency: f64) {
        self.emulator_cycle_interval = Duration::from_secs_f64(1.0 / frequency);
    }

    fn terminate(&mut self) {
        self.emulator_state = EmulatorState::Terminated;
    }

    fn reset(&mut self) {
        self.change_emulator(self.initial_snapshot.clone());
    }

    fn pause(&mut self) {
        self.emulator_state = EmulatorState::Paused;
        self.emulator.pause();
    }

    fn resume(&mut self, run_mode: RunMode) {
        self.emulator.resume();
        self.emulator_state = EmulatorState::Running(run_mode);
        self.next_emulator_cycle = Instant::now() + self.emulator_cycle_interval;
        self.next_render = Instant::now() + self.render_interval;
        self.next_emulator_frame = Instant::now() + self.emulator_frame_interval;
    }

    fn change_emulator(&mut self, emulator: Emulator) {
        self.emulator = emulator;
        self.history.clear();
    }

    pub fn load_rom(&mut self, bytes: &[u8]) -> Result<()> {
        let emu = Emulator::load_program(bytes).map_err(Chip8Error::ProgramLoadingFailed)?;
        self.change_emulator(emu);
        self.initial_snapshot = self.emulator.clone();
        Ok(())
    }

    pub(crate) fn handle_action(&mut self, action: &Action) {
        match (action, self.emulator_state) {
            (Action::Quit, _) => self.terminate(),

            (Action::Reset, EmulatorState::Running(_) | EmulatorState::Paused) => self.reset(),

            (Action::TogglePause, EmulatorState::Running(_)) => self.pause(),
            (Action::TogglePause, EmulatorState::Paused) => self.resume(RunMode::Standard),

            (Action::Step, EmulatorState::Running(RunMode::Standard)) => self.pause(),
            (Action::Step, EmulatorState::Paused) => {
                self.resume(RunMode::Step);
            }

            (Action::Chip8KeyPress(ch8_key), EmulatorState::Running(_)) => {
                self.emulator.press_key(*ch8_key);
            }

            (Action::_ChangeClockSpeed(frequency), _) => self.set_clock_speed(*frequency),

            (_, _) => {}
        }
    }

    /// Blocks until next event is available and process it
    fn process_next_event(&mut self) -> RunResult<()> {
        if let Some(action) =
            T::read_event().map_err(|e| RunError::CouldNotPollEvents(Box::new(e)))?
        {
            self.handle_action(&action);
        }
        Ok(())
    }

    /// Process available events
    fn process_events(&mut self) -> RunResult<()> {
        for action in T::poll_events().map_err(|e| RunError::CouldNotPollEvents(Box::new(e)))? {
            self.handle_action(&action);
        }
        Ok(())
    }

    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> RunResult<()> {
        loop {
            self.process_events()?;

            if Instant::now() > self.next_render {
                terminal
                    .draw(|frame| tui::draw(self, frame))
                    .map_err(RunError::RenderFailed)?;
                self.next_render += self.render_interval;
            }

            match self.emulator_state {
                EmulatorState::Terminated => break,
                EmulatorState::Running(run_mode) => {
                    self.run_emulator(self.next_render, run_mode)?;
                }
                EmulatorState::Paused => self.process_next_event()?,
            }
        }

        Ok(())
    }

    fn run_emulator(&mut self, until: Instant, mode: RunMode) -> RunResult<()> {
        loop {
            if Instant::now() > self.next_emulator_frame {
                self.emulator.decrement_timers();
                self.emulator.keyboard.release_keys();
                self.next_emulator_frame += self.emulator_frame_interval;
            }

            if Instant::now() >= self.next_emulator_cycle
                && self.emulator.cpu_state == CpuState::Executing
            {
                let pc = self.emulator.registers.program_counter;
                let last_instr = self.emulator.step()?;
                if self.emulator.registers.program_counter == pc {
                    // execution address has not changed, the program has entered a dead state
                    self.terminate();
                    break;
                }

                self.history.push(last_instr);
                self.next_emulator_cycle += self.emulator_cycle_interval;

                if mode == RunMode::Step {
                    self.pause();
                    break;
                }
            }

            std::thread::sleep(
                self.next_emulator_cycle
                    .min(self.next_emulator_frame)
                    .min(until)
                    .saturating_duration_since(Instant::now()),
            );

            if Instant::now() >= until {
                break;
            }
        }

        Ok(())
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
    CouldNotPollEvents(Box<dyn std::error::Error>),

    #[error("render failed: {0}")]
    RenderFailed(std::io::Error),
}
