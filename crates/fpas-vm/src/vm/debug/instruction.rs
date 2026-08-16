//! Feasibility rejection for stopped-state instruction-pointer changes.
//!
//! Existing bytecode verification proves the original control-flow graph from
//! function entry, register-index bounds, and sequence-point source mapping. It
//! does not retain per-instruction initialized-register sets or type
//! environments. Linear-scan allocation reuses temporary registers, so an
//! interior jump can resume with the wrong type or an uninitialized operand.
//! Function-entry reconstruction remains [`super::frame_restart`].
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::types::{DebugErrorKind, DebugSessionError};

/// Stable rejection for every instruction-pointer destination.
pub(super) fn unsupported() -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::InstructionChangeUnsupported,
        message: "arbitrary instruction changes are not supported".to_string(),
        hint: "Use stepping, a source breakpoint, or frame restart. The verifier cannot prove register initialization, operand types, or lexical state for an interior jump.".to_string(),
    }
}
