//! Orchestration: turns a parsed CLI command into a concrete PDF job.
//!
//! Each operation gets a real implementation in a later phase; the CLI
//! shell wires the command surface now and reports unsupported operations.

use crate::cli::Command;
use crate::error::{Error, Result};

pub fn run(command: Command) -> Result<()> {
    let label = match &command {
        Command::Compress { file } => format!("compress `{}`", file.display()),
        Command::Merge { files, .. } => format!("merge {} documents", files.len()),
        Command::Split { file } => format!("split `{}`", file.display()),
        Command::Remove { file } => format!("remove pages from `{}`", file.display()),
        Command::Extract { file } => format!("extract pages from `{}`", file.display()),
        Command::Password { file } => format!("set a password on `{}`", file.display()),
        Command::Info { file } => format!("read info for `{}`", file.display()),
    };
    Err(Error::Unsupported(format!(
        "{label} — not implemented yet (planned for a later phase)"
    )))
}