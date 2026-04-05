use std::path::PathBuf;

use crate::{
    cpu::{Cpu, RunError},
    memory::MemoryError,
};

#[derive(Debug)]
pub struct Interpreter {}

impl Interpreter {
    pub fn run(program: PathBuf, clock_speed: Option<u64>) -> Result<(), InterpreterError> {
        let program = std::fs::read(&program).map_err(|e| InterpreterError::Io {
            error: e,
            path: program,
        })?;

        let mut cpu =
            Cpu::load_program(program.as_slice()).map_err(InterpreterError::LoadingFailed)?;
        if let Some(clock_speed) = clock_speed {
            cpu = cpu.with_clock_speed(clock_speed);
        }

        ratatui::run(|terminal| cpu.run(terminal))?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InterpreterError {
    #[error("running error: {0}")]
    Run(#[from] RunError),

    #[error("could not load program: {0}")]
    LoadingFailed(MemoryError),

    #[error("{error}: {path}")]
    Io {
        error: std::io::Error,
        path: PathBuf,
    },
}
