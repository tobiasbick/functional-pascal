//! Detached failures with diagnostics that exclude peer payloads and credentials.

use std::fmt;

/// Local registration, admission or reply failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Invalid native configuration.
    InvalidArgument(&'static str),
    /// Invalid or externally resolved JSON schema.
    InvalidSchema(&'static str),
    /// A tool name is already registered.
    DuplicateTool,
    /// The catalog has no such tool.
    UnknownTool,
    /// Arguments fail the registered schema.
    InvalidArguments,
    /// A successful tool result fails its output schema.
    InvalidResult,
    /// Admission failed before accepting application work.
    LimitExceeded(&'static str, usize),
    /// Another owner holds the receive operation.
    Busy,
    /// The resource or response right is terminal.
    Closed,
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidArgument(_) => "Invalid MCP argument",
            Self::InvalidSchema(_) => "Invalid MCP tool schema",
            Self::DuplicateTool => "MCP tool name is already registered",
            Self::UnknownTool => "Unknown MCP tool",
            Self::InvalidArguments => "MCP tool arguments do not match the input schema",
            Self::InvalidResult => "MCP tool result does not match the output schema",
            Self::LimitExceeded(_, _) => "MCP resource limit exceeded",
            Self::Busy => "MCP receive operation already has an owner",
            Self::Closed => "MCP resource is closed",
        })
    }
}

impl std::error::Error for Fault {}
