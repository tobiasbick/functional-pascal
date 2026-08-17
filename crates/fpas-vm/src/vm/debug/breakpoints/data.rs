//! Global data-breakpoint requests and bounded session bindings.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use crate::vm::debug::location::DebugDataLocationIdentity;

/// Observed access that can stop a debug session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataBreakpointAccess {
    /// Any store to the watched global slot.
    Write,
    /// A store that leaves a different value than the previous stop.
    Change,
    /// Load observation; not implemented for the current identity subset.
    Read,
}

/// Requested data breakpoint using a durable location identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataBreakpoint {
    /// Location identity from `location.describe`.
    pub identity: DebugDataLocationIdentity,
    /// Requested access kind.
    pub access: DataBreakpointAccess,
}

/// One logical data breakpoint retained by a debug session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundDataBreakpoint {
    /// Stable session-local breakpoint identifier.
    pub id: u64,
    /// Original request.
    pub requested: DataBreakpoint,
    /// Whether this identity can currently produce a data stop.
    pub verified: bool,
    /// Actionable reason when `verified` is false.
    pub message: Option<String>,
}

impl BoundDataBreakpoint {
    /// Return whether this breakpoint can stop execution.
    #[must_use]
    pub const fn is_verified(&self) -> bool {
        self.verified
    }
}

impl DataBreakpointAccess {
    /// Stable protocol token for this access kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Write => "write",
            Self::Change => "change",
            Self::Read => "read",
        }
    }

    /// Parse a protocol access token.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "write" => Some(Self::Write),
            "change" => Some(Self::Change),
            "read" | "readWrite" => Some(Self::Read),
            _ => None,
        }
    }
}

pub(in crate::vm::debug) fn bind(id: u64, requested: DataBreakpoint) -> BoundDataBreakpoint {
    let (verified, message) = match (&requested.identity, requested.access) {
        (DebugDataLocationIdentity::Global { .. }, DataBreakpointAccess::Write)
        | (DebugDataLocationIdentity::Global { .. }, DataBreakpointAccess::Change) => (true, None),
        (DebugDataLocationIdentity::Global { .. }, DataBreakpointAccess::Read) => (
            false,
            Some(
                "Read data breakpoints are unsupported; watch write or change on a global."
                    .to_string(),
            ),
        ),
        (DebugDataLocationIdentity::FrameRegister { .. }, _) => (
            false,
            Some(
                "Frame-register identities are live-frame only and are not watchable.".to_string(),
            ),
        ),
    };
    BoundDataBreakpoint {
        id,
        requested,
        verified,
        message,
    }
}
