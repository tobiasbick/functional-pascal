//! Portable saved execution state for one suspended register task.

use fpas_bytecode::{FunctionId, FunctionInfo, FunctionValue, Value};

use crate::vm::debug::initializer_suppression::SourceInitializerTarget;
use crate::vm::frame::CallFrame;
use crate::vm::hosted::callbacks::CallbackContinuation;

/// Complete mutable register-machine state transferable between pool workers.
pub(in crate::vm) struct TaskState {
    /// Pending operation resumed before dispatch when a pool task yields its thread.
    pub suspension: Option<super::TaskSuspension>,
    /// Retry ownership, if this is an explicitly supervised child.
    pub supervision: Option<Box<super::supervision::SupervisedTask>>,
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
    pub suppressed_initializers: Vec<SourceInitializerTarget>,
    /// Partially completed hosted operations whose callbacks run on this task.
    pub callback_continuations: Vec<CallbackContinuation>,
}

impl TaskState {
    /// Build a fresh task with receiver, explicit arguments, then immutable captures.
    pub(in crate::vm::tasks) fn entry(
        id: u64,
        function: &FunctionValue,
        info: &FunctionInfo,
        arguments: impl IntoIterator<Item = Value>,
        retain_result: bool,
    ) -> Self {
        let (registers, register_initialized) = crate::vm::worker::Worker::register_window(
            usize::from(info.register_count),
            function
                .bound_receiver
                .iter()
                .cloned()
                .chain(arguments)
                .chain(function.captures.iter().cloned()),
        );
        Self {
            suspension: None,
            supervision: None,
            id,
            function: function.function,
            ip: info.code.start.get() as usize,
            base: 0,
            registers,
            register_initialized,
            frames: Vec::new(),
            retain_result,
            instruction_count: 0,
            suppressed_initializers: Vec::new(),
            callback_continuations: Vec::new(),
        }
    }
}
