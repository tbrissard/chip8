use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::app::App;

mod app;
mod cpu;
mod input;
mod keyboard;
mod memory;
mod screen;
mod tui;

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run {
        program: PathBuf,

        /// Number of instructions per second
        #[arg(long)]
        clock_speed: Option<f64>,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Run {
            program,
            clock_speed,
        } => {
            let mut app = App::default();
            let program = std::fs::read(program).unwrap();
            if let Some(frequency) = clock_speed {
                app.set_clock_speed(frequency);
            }
            app.load_program(&program).unwrap();
            ratatui::run(|terminal| app.run(terminal)).unwrap();
        }
    }
}
