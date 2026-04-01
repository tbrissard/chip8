use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Run { program: PathBuf },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Run { program } => {
            if let Err(e) = chip8::Interpreter::run(program) {
                println!("{e}");
            }
        }
    }
}
