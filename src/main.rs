use std::path::PathBuf;

use clap::{Parser, Subcommand};

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
        clock_speed: Option<u64>,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Run {
            program,
            clock_speed,
        } => {
            if let Err(e) = chip8::Interpreter::run(program, clock_speed) {
                println!("{e}");
            }
        }
    }
}
