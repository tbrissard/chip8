use std::{
    marker::PhantomData,
    sync::{
        Arc, Mutex,
        mpsc::{self, SendError, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use ratatui::DefaultTerminal;

use crate::{
    emulator::{Emulator, EmulatorMessage, MemoryError, StepError},
    input::InputManager,
    tui,
};

const FRAME_RATE: f64 = 60.0;

#[derive(Debug, Clone, Copy)]
pub(crate) enum Action {
    /// Exit the emulator
    Quit,
    /// Resets the loaded program
    Reset,
    /// Send a message to the emulator
    Message(EmulatorMessage),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum AppState {
    #[default]
    Running,
    Terminating,
}

#[derive(Debug)]
pub(crate) struct App<T> {
    state: AppState,
    pub(crate) emulator: Arc<Mutex<Emulator>>,
    input_manager: PhantomData<T>,

    initial_snapshot: Emulator,

    render_interval: Duration,
    next_render: Instant,
}

impl<T> Default for App<T> {
    fn default() -> Self {
        Self {
            state: AppState::default(),
            emulator: Arc::new(Mutex::new(Emulator::default())),
            input_manager: PhantomData,

            initial_snapshot: Emulator::default(),

            render_interval: Duration::from_secs_f64(1.0 / FRAME_RATE),
            next_render: Instant::now(),
        }
    }
}

impl<T: InputManager> App<T> {
    pub fn set_clock_speed(&mut self, frequency: f64) {
        self.emulator.lock().unwrap().cycle_interval = Duration::from_secs_f64(1.0 / frequency);
    }

    // fn reset(&mut self) {
    //     self.change_emulator(self.initial_snapshot.clone());
    // }

    // fn change_emulator(&mut self, emulator: Emulator) {
    //     self.emulator = emulator;
    // }

    pub fn load_rom(&mut self, bytes: &[u8]) -> Result<()> {
        let emu = Emulator::load_program(bytes).map_err(Chip8Error::ProgramLoadingFailed)?;
        self.initial_snapshot = emu.clone();
        self.emulator = Arc::new(Mutex::new(emu));
        Ok(())
    }

    pub(crate) fn handle_action(
        &mut self,
        action: Action,
        tx: &mut Sender<EmulatorMessage>,
    ) -> std::result::Result<(), SendError<EmulatorMessage>> {
        match action {
            Action::Quit => self.terminate(),
            Action::Reset => todo!(),
            Action::Message(emulator_message) => tx.send(emulator_message)?,
        }
        Ok(())
    }

    /// Process available events
    fn process_events(&mut self, tx: &mut Sender<EmulatorMessage>) -> RunResult<()> {
        for action in T::poll_events().map_err(|e| RunError::CouldNotPollEvents(Box::new(e)))? {
            self.handle_action(action, tx).expect("error: {e}");
        }
        Ok(())
    }

    pub(crate) fn run(&mut self, terminal: &mut DefaultTerminal) -> RunResult<()> {
        let (mut tx, rx) = mpsc::channel::<EmulatorMessage>();
        let emulator = self.emulator.clone();
        let handle = thread::spawn(move || emulator.lock().unwrap().run(rx));

        loop {
            self.process_events(&mut tx)?;

            if Instant::now() >= self.next_render {
                terminal
                    .draw(|frame| tui::draw(self, frame))
                    .map_err(RunError::RenderFailed)?;
                self.next_render += self.render_interval;
            }

            if self.state == AppState::Terminating {
                tx.send(EmulatorMessage::Stop).unwrap();
                handle.join().unwrap();
                break;
            }
        }

        Ok(())
    }

    fn terminate(&mut self) {
        self.state = AppState::Terminating;
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
