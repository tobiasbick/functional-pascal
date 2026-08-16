//! Safe register interpreter lifecycle, tasks, and concurrency.

mod access;
mod callback;
mod callback_call;
mod calls;
mod debug;
mod diagnostics;
mod dispatch;
mod execute;
mod frame;
mod hosted;
mod layouts;
mod register_stack;
mod shared;
mod tasks;
mod value_ops;
mod worker;

#[cfg(test)]
mod tests;

use std::sync::{Arc, RwLock};

use fpas_bytecode::{FunctionId, Value, VerifiedExecutable};

use self::hosted::HostedState;
use self::layouts::RuntimeLayouts;
use self::tasks::TaskScheduler;
use self::worker::Worker;

pub use callback::CallbackSession;
pub use debug::{
    BoundBreakpoint, BoundFunctionBreakpoint, DebugArrayMutationResult, DebugAssignmentSelector,
    DebugAssignmentTarget, DebugBinaryOperation, DebugBreakpointLimits,
    DebugDictionaryMutationResult, DebugErrorKind, DebugEvaluateResult,
    DebugEvaluationCancelHandle, DebugEvaluationLimits, DebugExecutionLimits, DebugExpression,
    DebugForcedReturnResult, DebugFrame, DebugInspectionLimits, DebugPauseHandle, DebugRunResult,
    DebugScope, DebugScopeKind, DebugSession, DebugSessionError, DebugSessionState, DebugStop,
    DebugStopReason, DebugStorageInitializationResult, DebugStringMutationResult, DebugTask,
    DebugTaskEvent, DebugTaskEventKind, DebugTaskResultReplacement, DebugTaskState,
    DebugTermination, DebugUnaryOperation, DebugVariable, DebugVariantConstructionResult,
    DebugVariantDescription, DebugVariantField, DebugVariantInfo, FunctionBreakpoint, Paginated,
    SourceBreakpoint, SourceLocation,
};
pub use diagnostics::VmError;
pub(crate) use diagnostics::runtime_error;

/// Captured console output produced by the virtual machine.
pub type VmOutput = fpas_std::CapturedOutput;

pub(crate) use shared::{GraphState, TaskBatchPoll, TaskResultPoll, TaskResultState, TaskTimers};

const TIMESLICE: u32 = 256;

/// Successful root execution result and diagnostic instruction count.
#[derive(Debug, Clone, PartialEq)]
pub struct Execution {
    /// Root or callback return value.
    pub value: Value,
    /// Number of packed instructions dispatched by this run.
    pub instruction_count: u64,
}

/// Single-use VM for a pre-verified executable.
///
/// This API is intentionally not wired to the production CLI until later phases complete
/// intrinsics, tasks, persistent artifacts, and the production cutover.
pub struct Vm {
    executable: Arc<VerifiedExecutable>,
    has_run: bool,
    globals: Arc<RwLock<Vec<Option<Value>>>>,
    layouts: Result<Arc<RuntimeLayouts>, VmError>,
    hosted: Arc<HostedState>,
    pool_size: usize,
    scheduler: Arc<TaskScheduler>,
}

impl Vm {
    /// Construct an isolated VM from an owned verified executable.
    #[must_use]
    pub fn new(executable: VerifiedExecutable) -> Self {
        Self::from_shared_with_host(Arc::new(executable), fpas_std::Console::new(), Vec::new())
    }

    /// Construct an isolated VM sharing immutable verified executable metadata.
    #[must_use]
    pub fn from_shared(executable: Arc<VerifiedExecutable>) -> Self {
        Self::from_shared_with_host(executable, fpas_std::Console::new(), Vec::new())
    }

    /// Construct an isolated VM with process arguments visible through `Std.Args`.
    #[must_use]
    pub fn with_args(executable: VerifiedExecutable, arguments: Vec<String>) -> Self {
        Self::from_shared_with_host(Arc::new(executable), fpas_std::Console::new(), arguments)
    }

    /// Construct an isolated VM that streams hosted console output.
    #[must_use]
    pub fn with_writer(
        executable: VerifiedExecutable,
        writer: Box<dyn std::io::Write + Send>,
    ) -> Self {
        Self::from_shared_with_host(
            Arc::new(executable),
            fpas_std::Console::with_writer(writer),
            Vec::new(),
        )
    }

    /// Construct an isolated VM that streams output and exposes process arguments.
    #[must_use]
    pub fn with_writer_and_args(
        executable: VerifiedExecutable,
        writer: Box<dyn std::io::Write + Send>,
        arguments: Vec<String>,
    ) -> Self {
        Self::from_shared_with_host(
            Arc::new(executable),
            fpas_std::Console::with_writer(writer),
            arguments,
        )
    }

    fn from_shared_with_host(
        executable: Arc<VerifiedExecutable>,
        console: fpas_std::Console,
        arguments: Vec<String>,
    ) -> Self {
        let globals = Arc::new(RwLock::new(vec![
            None;
            executable.executable().globals.len()
        ]));
        let layouts = RuntimeLayouts::build(
            executable.executable(),
            fpas_bytecode::InstructionAddress::new(0),
        )
        .map(Arc::new);
        let pool_size = if executable
            .executable()
            .functions
            .iter()
            .any(|function| function.flags.uses_spawn_tasks)
        {
            std::thread::available_parallelism()
                .map(|count| count.get())
                .unwrap_or(1)
                .saturating_sub(1)
                .max(1)
        } else {
            0
        };
        Self {
            executable,
            has_run: false,
            globals,
            layouts,
            hosted: Arc::new(HostedState::new(console, arguments)),
            pool_size,
            scheduler: Arc::new(TaskScheduler::new()),
        }
    }

    /// Queue one line for hosted `Read` and `ReadLn` calls.
    pub fn push_readln_input(&mut self, line: &str) {
        self.hosted
            .text_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push_line(line);
    }

    /// Queue characters for hosted `Std.Console.ReadKey` calls.
    pub fn push_readkey_input(&mut self, characters: &str) {
        self.hosted
            .key_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push_chars(characters);
    }

    /// Queue one structured hosted key event.
    pub fn push_key_event(&mut self, event: fpas_std::ConsoleKeyEvent) {
        self.hosted
            .key_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push_key_event(event);
    }

    /// Queue one unified hosted console event.
    pub fn push_console_event(&mut self, event: fpas_std::ConsoleEvent) {
        self.hosted
            .key_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push_console_event(event);
    }

    /// Queue one hosted graph event, retaining it until a session opens when necessary.
    pub fn push_graph_event(&mut self, event: fpas_std::GraphEvent) {
        let mut graph = self
            .hosted
            .graph
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if graph
            .session
            .push_event(event.clone(), fpas_bytecode::SourceLocation::new(1, 1))
            .is_err()
        {
            graph.pending_test_events.push(event);
        }
    }

    /// Return the currently captured console output.
    #[must_use]
    pub fn output(&self) -> VmOutput {
        self.hosted
            .console
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .output()
            .clone()
    }

    /// Return a deterministic logical console-screen snapshot.
    #[must_use]
    pub fn screen_snapshot(&self) -> fpas_std::ScreenSnapshot {
        self.hosted
            .console
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .screen_snapshot()
    }

    /// Return a thread-safe handle that cooperatively cancels this VM at an instruction boundary.
    #[must_use]
    pub fn shutdown_handle(&self) -> ShutdownHandle {
        ShutdownHandle {
            scheduler: Arc::clone(&self.scheduler),
        }
    }

    /// Execute the verified root function once.
    ///
    /// # Errors
    ///
    /// Returns the preserved runtime diagnostic for scalar failures, or an internal invariant
    /// diagnostic if a verified executable requests an opcode assigned to a later phase.
    pub fn run(&mut self) -> Result<Execution, VmError> {
        if std::mem::replace(&mut self.has_run, true) {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                fpas_bytecode::InstructionAddress::new(0),
                fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN,
                "This VM instance has already been run",
                "Register VM instances are single-use. Construct a new instance for each run.",
            ));
        }
        let scheduler = Arc::clone(&self.scheduler);
        let worker = Worker::for_function_with_state(
            Arc::clone(&self.executable),
            self.executable.executable().entry,
            Vec::new(),
            Arc::clone(&self.globals),
            self.layouts.clone()?,
            Arc::clone(&self.hosted),
        )?
        .with_scheduler(Some(Arc::clone(&scheduler)));
        if self.pool_size == 0 {
            return worker.run();
        }
        let pool_size = self.pool_size;
        std::thread::scope(|scope| {
            let mut handles = Vec::with_capacity(pool_size);
            for _ in 0..pool_size {
                let scheduler = Arc::clone(&scheduler);
                let template = worker.pool_template();
                handles.push(scope.spawn(move || tasks::pool::pool_loop(&template, scheduler)));
            }
            let timer_scheduler = Arc::clone(&scheduler);
            let timer = scope.spawn(move || timer_scheduler.timer_loop());
            let main = worker.run();
            if let Err(error) = &main {
                scheduler.fail(error.clone());
            } else {
                scheduler.finish_main();
            }
            let mut pool_error = None;
            for handle in handles {
                match handle.join() {
                    Ok(Err(error)) if pool_error.is_none() => pool_error = Some(error),
                    Err(_) if pool_error.is_none() => {
                        pool_error = scheduler.first_error().or_else(|| {
                            Some(diagnostics::internal(
                                self.executable.executable(),
                                fpas_bytecode::InstructionAddress::new(0),
                                "Register task worker panicked",
                            ))
                        });
                    }
                    _ => {}
                }
            }
            if timer.join().is_err() && pool_error.is_none() {
                pool_error = Some(diagnostics::internal(
                    self.executable.executable(),
                    fpas_bytecode::InstructionAddress::new(0),
                    "Register task timer panicked",
                ));
            }
            main.and_then(|value| pool_error.map_or(Ok(value), Err))
        })
    }

    /// Execute one numeric function as a synchronous hosted callback.
    ///
    /// # Errors
    ///
    /// Returns a runtime diagnostic for an invalid target, wrong arity, shutdown reuse, panic, or
    /// resource-limit failure. No function name lookup occurs.
    pub fn call(
        &mut self,
        function: FunctionId,
        arguments: Vec<Value>,
    ) -> Result<Execution, VmError> {
        if std::mem::replace(&mut self.has_run, true) {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                fpas_bytecode::InstructionAddress::new(0),
                fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN,
                "This VM instance has already been run",
                "Register VM instances are single-use. Construct a new instance for each callback.",
            ));
        }
        Worker::for_function_with_state(
            Arc::clone(&self.executable),
            function,
            arguments,
            Arc::clone(&self.globals),
            self.layouts.clone()?,
            Arc::clone(&self.hosted),
        )?
        .run()
    }
}

/// Cloneable cooperative-cancellation handle for a running [`Vm`].
#[derive(Clone)]
pub struct ShutdownHandle {
    scheduler: Arc<TaskScheduler>,
}

impl ShutdownHandle {
    /// Request cancellation. The main and spawned tasks stop at their next instruction boundary.
    pub fn shutdown(&self) {
        self.scheduler.request_cancel();
    }
}
