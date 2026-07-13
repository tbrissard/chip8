use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use ratatui::DefaultTerminal;

use crate::{
    emulator::{CpuState, Emulator, Instruction, MemoryError, Registers, StepError},
    input,
    keyboard::{Ch8Key, Ch8Keyboard},
    screen::StandardScreen,
    tui,
};

pub type Result<T> = std::result::Result<T, Chip8Error>;

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
enum EmulatorState {
    #[default]
    Running,
    Stepping,
    Paused,
    Terminated,
}

#[derive(Debug)]
pub(crate) struct App {
    history: Vec<Instruction>,
    pub(crate) emulator: Emulator,

    emulator_state: EmulatorState,
    initial_snapshot: Emulator,

    tick_interval_app: Duration,
    next_tick_app: Instant,
    frame_interval: Duration,
    next_frame: Instant,
    tick_interval_timer: Duration,
    next_tick_timer: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            history: Vec::new(),
            emulator: Emulator::default(),

            emulator_state: EmulatorState::default(),
            initial_snapshot: Emulator::default(),

            tick_interval_app: Duration::from_secs_f64(1.0 / DEFAULT_CLOCK_SPEED),
            next_tick_app: Instant::now(),
            frame_interval: Duration::from_secs_f64(1.0 / FRAME_RATE),
            next_frame: Instant::now(),
            tick_interval_timer: Duration::from_secs_f64(1.0 / TIMER_TICK_RATE),
            next_tick_timer: Instant::now(),
        }
    }
}

impl App {
    pub fn set_clock_speed(&mut self, frequency: f64) {
        self.tick_interval_app = Duration::from_secs_f64(1.0 / frequency)
    }

    fn terminate(&mut self) {
        self.emulator_state = EmulatorState::Terminated
    }

    fn reset(&mut self) {
        self.sets_emulator_state(self.initial_snapshot.clone());
    }

    fn pause(&mut self) {
        self.emulator_state = EmulatorState::Paused
    }

    fn resume(&mut self) {
        self.emulator_state = EmulatorState::Running;
        self.next_tick_app = Instant::now() + self.tick_interval_app;
        self.next_frame = Instant::now() + self.frame_interval;
        self.next_tick_timer = Instant::now() + self.tick_interval_timer;
    }

    fn step(&mut self) {
        self.emulator_state = EmulatorState::Stepping;
    }

    fn sets_emulator_state(&mut self, emulator: Emulator) {
        self.emulator = emulator;
        self.history.clear();
    }

    pub fn load_rom(&mut self, bytes: &[u8]) -> Result<()> {
        let emu = Emulator::load_program(bytes).map_err(Chip8Error::ProgramLoadingFailed)?;
        self.sets_emulator_state(emu);
        self.initial_snapshot = self.emulator.clone();
        Ok(())
    }

    pub(crate) fn handle_action(&mut self, action: &Action) -> Result<()> {
        match (action, self.emulator_state) {
            (Action::Quit, _) => self.terminate(),

            (
                Action::Reset,
                EmulatorState::Running | EmulatorState::Stepping | EmulatorState::Paused,
            ) => self.reset(),
            (Action::Reset, EmulatorState::Terminated) => {}

            (Action::TogglePause, EmulatorState::Running | EmulatorState::Stepping) => self.pause(),
            (Action::TogglePause, EmulatorState::Paused) => self.resume(),
            (Action::TogglePause, _) => {}

            (Action::Step, EmulatorState::Running | EmulatorState::Paused) => self.step(),
            (Action::Step, _) => {}

            (Action::Chip8KeyPress(ch8_key), EmulatorState::Running | EmulatorState::Stepping) => {
                self.emulator.press_key(*ch8_key)
            }
            (Action::Chip8KeyPress(_), _) => {}

            (Action::_ChangeClockSpeed(frequency), _) => self.set_clock_speed(*frequency),
        }
        Ok(())
    }

    pub(crate) fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> std::result::Result<(), RunError> {
        while self.emulator_state != EmulatorState::Terminated {
            for a in input::poll_action().map_err(RunError::ActionPollFailed)? {
                self.handle_action(&a).map_err(|e| RunError::Action(a, e))?;
            }

            if matches!(
                self.emulator_state,
                EmulatorState::Running | EmulatorState::Stepping
            ) {
                if self.emulator.cpu_state == CpuState::Running {
                    let pc = self.emulator.registers.program_counter;

                    let last_instr = self.emulator.step()?;
                    self.history.push(last_instr);

                    if self.emulator.registers.program_counter == pc {
                        self.terminate();
                    };
                }

                if Instant::now() > self.next_tick_timer {
                    self.emulator.decrease_delay_timer();
                    self.emulator.decrease_sound_timer();
                    self.next_tick_timer += self.tick_interval_timer;
                    self.emulator.keyboard.release_keys();
                }

                if matches!(self.emulator_state, EmulatorState::Stepping) {
                    self.pause();
                }
            }

            if Instant::now() > self.next_frame {
                terminal
                    .draw(|frame| tui::draw(self, frame))
                    .map_err(RunError::RenderFailed)?;
                self.next_frame += self.frame_interval;
            }

            std::thread::sleep(self.next_tick_app.saturating_duration_since(Instant::now()));
            self.next_tick_app += self.tick_interval_app;
        }

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

#[derive(Debug, thiserror::Error)]
pub enum Chip8Error {
    #[error("could not load program: {0}")]
    ProgramLoadingFailed(MemoryError),
}

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("could not handle {0}: {1}")]
    Action(Action, Chip8Error),

    #[error("emulator tick failed: {0}")]
    Execution(#[from] StepError),

    #[error("could not poll actions: {0}")]
    ActionPollFailed(std::io::Error),

    #[error("render failed: {0}")]
    RenderFailed(std::io::Error),
}
