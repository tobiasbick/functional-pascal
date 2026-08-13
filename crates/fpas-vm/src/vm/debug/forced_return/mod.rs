//! Protocol-neutral forced return from the active ordinary callee.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

mod commit;
mod validate;

use super::inspection::DebugFrame;
use super::types::{DebugErrorKind, DebugSessionError};

pub(in crate::vm::debug) use commit::commit;
pub(in crate::vm::debug) use validate::{
    prepare_return_value, reject_declared_category, require_convention, require_eligible,
    require_result_type,
};

/// Rendered result of one successful forced return.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugForcedReturnResult {
    /// Task that owned the completed callee.
    pub task_id: u64,
    /// Bounded FPAS value summary of the forced result.
    pub value: String,
    /// Runtime or source type name of the forced result.
    pub type_name: String,
    /// Stop-local reference for aggregate expansion, or zero for a leaf.
    pub variables_reference: u64,
    /// Number of named children.
    pub named_variables: usize,
    /// Number of indexed children.
    pub indexed_variables: usize,
    /// Fresh caller frame after the one-frame unwind.
    pub frame: DebugFrame,
}

pub(super) fn unsupported(
    message: impl Into<String>,
    hint: impl Into<String>,
) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::FrameReturnUnsupported,
        message: message.into(),
        hint: hint.into(),
    }
}

pub(super) fn value_required() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::FrameReturnValueRequired,
        message: "forced return requires a return expression for this function".to_string(),
        hint: "Supply one FPAS expression that evaluates to the declared result type, for example `42`."
            .to_string(),
    }
}

pub(super) fn value_unexpected() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::FrameReturnValueUnexpected,
        message: "forced return does not accept a return expression for this procedure".to_string(),
        hint: "Omit `expression` when completing a procedure; procedures return `unit`."
            .to_string(),
    }
}

pub(super) fn unknown_frame(frame_id: u64) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::UnknownFrame,
        message: format!("debug frame {frame_id} is unknown or expired"),
        hint: "Request stack frames again for the current stop.".to_string(),
    }
}
