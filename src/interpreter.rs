use std::path::PathBuf;

use crate::cpu::{Cpu, ExecutionError};

#[derive(Debug)]
pub struct Interpreter {}

impl Interpreter {
    pub fn run(program: PathBuf, clock_speed: Option<u64>) -> Result<(), InterpreterError> {
        let program = std::fs::read(&program).map_err(|e| InterpreterError::Io {
            error: e,
            path: program,
        })?;

        let mut cpu = Cpu::load_program(program.as_slice())?;
        if let Some(clock_speed) = clock_speed {
            cpu = cpu.with_clock_speed(clock_speed);
        }

        ratatui::run(|terminal| cpu.run(terminal))?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InterpreterError {
    #[error("execution error: {0}")]
    Execution(#[from] ExecutionError),

    #[error("{error}: {path}")]
    Io {
        error: std::io::Error,
        path: PathBuf,
    },
}
