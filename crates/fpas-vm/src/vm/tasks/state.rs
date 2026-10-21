//! Portable saved execution state for one suspended register task.

use fpas_bytecode::{FunctionId, FunctionInfo, FunctionValue, MemoryDomain, Value};

use crate::vm::debug::initializer_suppression::SourceInitializerTarget;
use crate::vm::frame::CallFrame;
use crate::vm::hosted::callbacks::CallbackContinuation;

/// Complete mutable register-machine state transferable between pool workers.
pub(in crate::vm) struct TaskState {
    /// Stable logical-task ownership, independent of scheduler task numbering.
    pub domain: MemoryDomain,
    /// Optional catalog ownership, moved out before storing this state in its record.
    pub(in crate::vm) capture_record:
        Option<std::sync::Arc<crate::vm::failure_capture::catalog::TaskRecord>>,
    /// Child failure already observed by this task, retained through cleanup suspension.
    pub observed_task_failure: Option<u64>,
    /// Terminal body value/error retained while its owner cleanup suspends.
    pub pending_completion: Option<Result<Value, crate::vm::VmError>>,
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
    /// Task scopes opened by the saved call frames, innermost last.
    pub task_scopes: Vec<super::scopes::OpenScope>,
    /// Shared control of the task scope that owns this task, if any.
    pub owner_scope: Option<std::sync::Arc<super::scopes::ScopeControl>>,
    pub execution: Option<fpas_bytecode::ExecutionRoots>,
}

impl TaskState {
    /// Build a fresh task with receiver, explicit arguments, then immutable captures.
    #[cfg(test)]
    pub(in crate::vm::tasks) fn entry(
        id: u64,
        function: &FunctionValue,
        info: &FunctionInfo,
        arguments: impl IntoIterator<Item = Value>,
    ) -> Self {
        Self::entry_in_domain(
            id,
            function,
            info,
            arguments,
            crate::vm::domain::new_domain(),
        )
    }

    pub(in crate::vm::tasks) fn entry_in_domain(
        id: u64,
        function: &FunctionValue,
        info: &FunctionInfo,
        arguments: impl IntoIterator<Item = Value>,
        domain: MemoryDomain,
    ) -> Self {
        let execution = domain.execution_roots().ok();
        let (registers, register_initialized) = crate::vm::worker::Worker::register_window(
            usize::from(info.register_count),
            function
                .bound_receiver
                .iter()
                .cloned()
                .chain(arguments)
                .chain(function.captures.iter().cloned()),
        );
        let state = Self {
            execution,
            domain,
            capture_record: None,
            observed_task_failure: None,
            pending_completion: None,
            suspension: None,
            supervision: None,
            id,
            function: function.function,
            ip: info.code.start.get() as usize,
            base: 0,
            registers,
            register_initialized,
            frames: Vec::new(),
            retain_result: true,
            instruction_count: 0,
            suppressed_initializers: Vec::new(),
            callback_continuations: Vec::new(),
            task_scopes: Vec::new(),
            owner_scope: None,
        };
        state.publish_execution();
        state
    }

    /// Publish before taking any scheduler/catalog storage lock. Incomplete views stay busy.
    pub(in crate::vm) fn publish_execution(&self) {
        use fpas_bytecode::TraceRoots;
        if let Some(execution) = &self.execution {
            let mut values = Vec::new();
            if self
                .trace_roots(&mut |value| values.push(value.clone()))
                .is_ok()
            {
                execution.publish(values);
                self.domain.request_collection_at_rest();
            } else {
                execution.invalidate();
            }
        }
    }
}
