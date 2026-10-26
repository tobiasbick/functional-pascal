//! Detached listener and executor failures without access credentials.

use crate::Fault;
use std::{fmt, io};

/// Local HTTP publication or shutdown failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HttpError {
    /// Registration or endpoint admission failed.
    Server(Fault),
    /// Listener setup or acceptance failed.
    Io(io::ErrorKind),
    /// The operating system could not supply access credentials.
    Entropy,
    /// The listener task failed.
    Executor,
    /// Connections did not stop within the configured shutdown deadline.
    ShutdownTimeout,
}

impl fmt::Display for HttpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Server(_) => "MCP HTTP endpoint is unavailable",
            Self::Io(_) => "MCP HTTP listener failed",
            Self::Entropy => "MCP HTTP credentials are unavailable",
            Self::Executor => "MCP HTTP executor failed",
            Self::ShutdownTimeout => "MCP HTTP shutdown deadline exceeded",
        })
    }
}

impl std::error::Error for HttpError {}

impl From<Fault> for HttpError {
    fn from(fault: Fault) -> Self {
        Self::Server(fault)
    }
}

impl From<io::Error> for HttpError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.kind())
    }
}
