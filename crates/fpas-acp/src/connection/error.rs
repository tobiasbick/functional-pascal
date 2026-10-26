//! Detached operational failures; implicit diagnostics contain no peer payload.

use crate::RawJson;
use std::fmt;

/// Connection/transport failure without a VM or callback dependency.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Invalid caller-supplied configuration or payload.
    InvalidArgument(&'static str),
    /// The peer has not advertised the optional operation.
    UnsupportedCapability(&'static str),
    /// The initialization offer or response selects an unsupported version.
    UnsupportedVersion(u16),
    /// Resource admission failed before publication.
    LimitExceeded(&'static str, usize),
    /// Invalid JSON, UTF-8, protocol envelope or typed payload.
    InvalidMessage,
    /// Another reader or operation already owns the operation.
    Busy(&'static str),
    /// Closed or foreign identity.
    Closed,
    /// The local wait was cancelled.
    Cancelled,
    /// The local wait reached its deadline.
    Timeout,
    /// The peer returned a JSON-RPC error, retained as original JSON.
    Peer(RawJson),
    /// The host failed to start or service owned work.
    Host,
}

impl fmt::Display for Fault {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidArgument(_) => "Invalid ACP argument",
            Self::UnsupportedCapability(_) => "Unsupported ACP capability",
            Self::UnsupportedVersion(_) => "Unsupported ACP version",
            Self::LimitExceeded(_, _) => "ACP resource limit exceeded",
            Self::InvalidMessage => "Invalid ACP message",
            Self::Busy(_) => "ACP operation is busy",
            Self::Closed => "ACP connection is closed",
            Self::Cancelled => "ACP wait cancelled",
            Self::Timeout => "ACP wait timed out",
            Self::Peer(_) => "ACP peer error",
            Self::Host => "ACP host operation failed",
        })
    }
}

impl std::error::Error for Fault {}

impl From<crate::CodecError> for Fault {
    fn from(error: crate::CodecError) -> Self {
        if error.kind == crate::ErrorKind::LimitExceeded {
            Self::LimitExceeded("MessageBytes", crate::raw::MAX_RAW_BYTES)
        } else {
            Self::InvalidMessage
        }
    }
}
