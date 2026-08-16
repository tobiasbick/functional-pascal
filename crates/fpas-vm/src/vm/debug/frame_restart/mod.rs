//! Protocol-neutral restart of one selected live frame.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

mod prepare;

use super::inspection::DebugFrame;
use super::types::{DebugErrorKind, DebugSessionError};

pub(in crate::vm::debug) use prepare::{apply, prepare};

/// Result of restarting one selected frame at its verified function entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugFrameRestartResult {
    /// Task that owns the restarted frame.
    pub task_id: u64,
    /// Fresh active frame after stopped snapshots were rebuilt.
    pub frame: DebugFrame,
    /// Number of younger frames discarded by the restart.
    pub discarded_frames: usize,
}

pub(super) fn unsupported(
    message: impl Into<String>,
    hint: impl Into<String>,
) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::FrameRestartUnsupported,
        message: message.into(),
        hint: hint.into(),
    }
}
