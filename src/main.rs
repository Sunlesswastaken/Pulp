#![forbid(unsafe_code)]

mod cli;
mod compression;
mod core;
mod error;
mod operations;
mod pdf;
mod tui;

use std::io::IsTerminal;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> error::Result<()> {
    use clap::{CommandFactory, Parser};

    let cli = cli::Cli::parse();

    match cli.command {
        Some(command) => operations::run(command),
        None if !std::io::stdout().is_terminal() => {
            let mut command = cli::Cli::command();
            command.print_help().map_err(error::Error::from)?;
            println!();
            Ok(())
        }
        None => tui::run(),
    }
}