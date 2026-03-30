//! Per-thread worker that executes bytecode for one task at a time.
//!
//! **Documentation:** `docs/future/parallel-vm.md`

use super::diagnostics::{STACK_OVERFLOW_CODE, VmError};
use super::shared::{SharedState, TaskState};
use super::{CallFrame, STACK_MAX, TIMESLICE};
use fpas_bytecode::{SourceLocation, Value};
use std::sync::Arc;

/// A worker runs on a single OS thread and executes tasks pulled from the shared queue.
///
/// The main program (task 0) is executed by the main worker directly.
/// Additional workers are spawned in a thread pool to handle `go`-spawned tasks.
pub(crate) struct Worker {
    pub shared: Arc<SharedState>,
    pub ip: usize,
    pub current_location: SourceLocation,
    pub stack: Vec<Value>,
    pub call_stack: Vec<CallFrame>,
    pub current_task_id: u64,
    pub current_task_retain_result: bool,
    pub instructions_until_yield: u32,
    pub sync_call_depth: u32,
}

impl Worker {
    /// Create a new worker for the main task (task 0, starts at ip=0).
    pub fn new_main(shared: Arc<SharedState>) -> Self {
        Self {
            shared,
            ip: 0,
            current_location: SourceLocation::new(1, 1),
            stack: Vec::with_capacity(256),
            call_stack: Vec::new(),
            current_task_id: 0,
            current_task_retain_result: false,
            instructions_until_yield: TIMESLICE,
            sync_call_depth: 0,
        }
    }

    /// Create a worker for the thread pool (starts idle, picks tasks from queue).
    pub fn new_pool(shared: Arc<SharedState>) -> Self {
        Self {
            shared,
            ip: 0,
            current_location: SourceLocation::new(1, 1),
            stack: Vec::new(),
            call_stack: Vec::new(),
            current_task_id: u64::MAX, // sentinel — no task loaded
            current_task_retain_result: false,
            instructions_until_yield: TIMESLICE,
            sync_call_depth: 0,
        }
    }

    /// Load a task's saved state into this worker.
    pub fn load_task(&mut self, task: TaskState) {
        self.current_task_id = task.id;
        self.ip = task.ip;
        self.stack = task.stack;
        self.call_stack = task.call_stack;
        self.current_task_retain_result = task.retain_result;
        self.instructions_until_yield = TIMESLICE;
    }

    /// Save the current task's state so it can be enqueued for later resumption.
    pub fn save_task(&mut self) -> TaskState {
        TaskState {
            id: self.current_task_id,
            ip: self.ip,
            stack: std::mem::take(&mut self.stack),
            call_stack: std::mem::take(&mut self.call_stack),
            retain_result: self.current_task_retain_result,
        }
    }

    /// Push a value onto this worker's stack.
    pub(crate) fn push(&mut self, value: Value) -> Result<(), VmError> {
        if self.stack.len() >= STACK_MAX {
            return Err(super::runtime_error(
                STACK_OVERFLOW_CODE,
                "Stack overflow",
                "Reduce recursion depth or intermediate stack usage in this expression.",
                self.current_location,
            ));
        }
        self.stack.push(value);
        Ok(())
    }

    /// Pop a value from this worker's stack.
    pub(crate) fn pop(&mut self, location: SourceLocation) -> Result<Value, VmError> {
        self.stack.pop().ok_or_else(|| {
            super::internal_error(
                "Stack underflow",
                "This indicates a compiler/runtime stack layout bug. Please report it.",
                location,
            )
        })
    }

    /// Peek at the top of the stack without removing.
    pub(crate) fn peek(&self, location: SourceLocation) -> Result<&Value, VmError> {
        self.stack.last().ok_or_else(|| {
            super::internal_error(
                "Stack underflow on peek",
                "This indicates a compiler/runtime bug. Please report it.",
                location,
            )
        })
    }

    /// Whether a value is truthy for branch instructions.
    pub(crate) fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => *b,
            Value::Integer(n) => *n != 0,
            Value::Unit => false,
            Value::OptionNone => false,
            _ => true,
        }
    }

    /// Pool-worker main loop: pull tasks from the shared queue and execute them.
    ///
    /// Returns when shutdown is signalled and no tasks remain.
    pub fn pool_loop(&mut self) -> Result<(), VmError> {
        loop {
            // Try to get a task from the queue.
            if let Some(task) = self.shared.try_dequeue_task() {
                self.load_task(task);
                self.run_current_task()?;
            } else if self.shared.is_shutdown() {
                return Ok(());
            } else {
                // Wait for task or shutdown signal.
                let mut queue = self
                    .shared
                    .task_queue
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                // Re-check after acquiring lock.
                if let Some(task) = queue.pop() {
                    drop(queue);
                    self.load_task(task);
                    self.run_current_task()?;
                } else if self.shared.is_shutdown() {
                    return Ok(());
                } else {
                    let _guard = self
                        .shared
                        .task_available
                        .wait(queue)
                        .unwrap_or_else(|e| e.into_inner());
                }
            }
        }
    }

    /// Execute the currently loaded task until it completes or yields.
    ///
    /// `run()` stores the task result internally when a non-main task's
    /// top-level function returns. This method only needs to wake waiters.
    fn run_current_task(&mut self) -> Result<(), VmError> {
        match self.run() {
            Ok(()) => {
                // Wake workers that might be waiting for this task's result.
                self.shared.task_available.notify_all();
                Ok(())
            }
            Err(e) => {
                // Task errored — signal global shutdown.
                self.shared.request_shutdown();
                Err(e)
            }
        }
    }
}
