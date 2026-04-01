use std::path::PathBuf;

use crate::cpu::{Cpu, ExecutionError};

#[derive(Debug)]
pub struct Interpreter {}

impl Interpreter {
    pub fn run(program: PathBuf) -> Result<(), InterpreterError> {
        let program = std::fs::read(program)?;
        let mut cpu = Cpu::load_program(program.as_slice())?;
        ratatui::run(|terminal| cpu.run(terminal))?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InterpreterError {
    #[error("execution error: {0}")]
    Execution(#[from] ExecutionError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
