//! Register-window call-frame state and resource limits.

use fpas_bytecode::FunctionId;

/// Maximum nested calls accepted by the register interpreter.
pub(super) const MAX_CALL_DEPTH: usize = 4096;

/// Maximum live register slots across all active frames.
pub(super) const MAX_REGISTER_SLOTS: usize = 1_048_576;

/// Continuation saved while a callee owns the active register window.
#[derive(Debug, Clone, Copy)]
pub(super) struct CallFrame {
    pub function: FunctionId,
    pub ip: usize,
    pub base: usize,
    pub return_destination: Option<usize>,
}
