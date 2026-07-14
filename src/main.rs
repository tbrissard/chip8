#![warn(clippy::pedantic)]

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::{app::App, input::CrosstermInputManager};

mod app;
mod emulator;
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
        rom: PathBuf,

        /// Number of instructions per second
        #[arg(long)]
        clock_speed: Option<f64>,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Run { rom, clock_speed } => {
            let mut app = App::<CrosstermInputManager>::default();
            let bytes = std::fs::read(rom).unwrap();
            app.load_rom(&bytes).unwrap();
            clock_speed.inspect(|frequency| app.set_clock_speed(*frequency));
            ratatui::run(|terminal| app.run(terminal)).unwrap();
        }
    }
}
