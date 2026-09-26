//! Register-window call-frame state and resource limits.

use fpas_bytecode::FunctionId;

/// Maximum nested calls accepted by the register interpreter.
pub(super) const MAX_CALL_DEPTH: usize = 4096;

/// Maximum live register slots across all active frames.
pub(super) const MAX_REGISTER_SLOTS: usize = 1_048_576;

/// Continuation saved while a callee owns the active register window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CallFrame {
    pub function: FunctionId,
    pub ip: usize,
    pub base: usize,
    pub return_destination: Option<usize>,
    /// Caller's active register count at the call; restored on return because an overlapping
    /// callee frame starts inside the caller's argument window.
    pub frame_end: usize,
}
