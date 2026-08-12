//! Portable saved execution state for one suspended register task.

use fpas_bytecode::{FunctionId, Value};

use crate::vm::frame::CallFrame;

/// Complete mutable register-machine state transferable between pool workers.
pub(in crate::vm) struct TaskState {
    pub id: u64,
    pub function: FunctionId,
    pub ip: usize,
    pub base: usize,
    pub registers: Vec<Value>,
    /// Parallel initialized/uninitialized bits for `registers`.
    pub register_initialized: Vec<bool>,
    pub frames: Vec<CallFrame>,
    pub retain_result: bool,
    pub instruction_count: u64,
}
