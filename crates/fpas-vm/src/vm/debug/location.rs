//! Durable observable data identities independent of one inspection generation.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::inspection::{InspectionSnapshot, MutationRoot, MutationTarget};

/// Kind of a stopped-state data location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugDataLocationKind {
    /// Executable-global slot.
    Global,
    /// Register in one live call frame.
    FrameRegister,
    /// Shared mutable capture cell without an alias registry.
    ClosureCell,
}

/// How long a described location remains a valid identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugDataLocationLifetime {
    /// Survives continue for the rest of the debug session.
    Executable,
    /// Valid only while that call-frame activation is live.
    LiveFrame,
    /// Cell pointer identity exists, but aliases are not registered.
    UnregisteredAlias,
}

/// Structured identity that does not depend on display text or inspection handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugDataLocationIdentity {
    /// Global slot in the current executable.
    Global {
        /// Zero-based executable global index.
        index: u64,
    },
    /// Register window of one live activation.
    FrameRegister {
        /// Owning debug task.
        task_id: u64,
        /// Function operand identity.
        function: u64,
        /// Absolute worker register index.
        register: u64,
    },
}

/// One named location derived from a current-stop mutation target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugDataLocation {
    /// Root kind of the location.
    pub kind: DebugDataLocationKind,
    /// Proven lifetime of this identity.
    pub lifetime: DebugDataLocationLifetime,
    /// Whether the target names a descendant of the root.
    pub descendant: bool,
    /// Display-text-free identity, when the location can be named.
    pub identity: Option<DebugDataLocationIdentity>,
}

impl DebugDataLocationKind {
    /// Stable protocol token for this kind.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::FrameRegister => "frame_register",
            Self::ClosureCell => "closure_cell",
        }
    }
}

impl DebugDataLocationLifetime {
    /// Stable protocol token for this lifetime.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Executable => "executable",
            Self::LiveFrame => "live_frame",
            Self::UnregisteredAlias => "unregistered_alias",
        }
    }
}

/// Convert a stop-local mutation target into a generation-independent location.
pub(in crate::vm::debug) fn describe_target(
    target: &MutationTarget,
    task_id: u64,
    snapshot: &InspectionSnapshot,
) -> DebugDataLocation {
    let descendant = !target.path.is_empty();
    match &target.root {
        MutationRoot::Global(index) => DebugDataLocation {
            kind: DebugDataLocationKind::Global,
            lifetime: DebugDataLocationLifetime::Executable,
            descendant,
            identity: Some(DebugDataLocationIdentity::Global {
                index: u64::try_from(*index).unwrap_or(u64::MAX),
            }),
        },
        MutationRoot::FrameRegister(register) => DebugDataLocation {
            kind: DebugDataLocationKind::FrameRegister,
            lifetime: DebugDataLocationLifetime::LiveFrame,
            descendant,
            identity: snapshot.frame_function(target.frame_id).map(|function| {
                DebugDataLocationIdentity::FrameRegister {
                    task_id,
                    function: u64::from(function.get()),
                    register: u64::try_from(*register).unwrap_or(u64::MAX),
                }
            }),
        },
        MutationRoot::ClosureCell(_) => DebugDataLocation {
            kind: DebugDataLocationKind::ClosureCell,
            lifetime: DebugDataLocationLifetime::UnregisteredAlias,
            descendant,
            identity: None,
        },
    }
}
