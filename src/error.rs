use std::io;

use thiserror::Error;

/// Result alias used across the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// The single error type for pulp.
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("unsupported: {0}")]
    Unsupported(String),

    #[allow(dead_code)] // constructed once the compression/security backends land
    #[error("external tool `{tool}` failed: {reason}")]
    ExternalTool { tool: String, reason: String },
}
