//! Safe register interpreter lifecycle, tasks, and concurrency.

mod access;
mod callback;
mod callback_call;
mod calls;
mod cancellation;
mod channels;
mod debug;
mod diagnostics;
mod dispatch;
mod execute;
mod failure_capture;
mod frame;
mod globals;
mod hosted;
mod inspection;
mod layouts;
mod register_stack;
mod run;
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
    BoundBreakpoint, BoundDataBreakpoint, BoundFunctionBreakpoint, DataBreakpoint,
    DataBreakpointAccess, DebugArrayMutationResult, DebugAssignmentSelector, DebugAssignmentTarget,
    DebugBinaryOperation, DebugBreakpointLimits, DebugDataLocation, DebugDataLocationIdentity,
    DebugDataLocationKind, DebugDataLocationLifetime, DebugDictionaryMutationResult,
    DebugErrorKind, DebugEvaluateResult, DebugEvaluationCancelHandle, DebugEvaluationLimits,
    DebugExecutionLimits, DebugExpression, DebugForcedReturnResult, DebugFrame,
    DebugFrameRestartResult, DebugInspectionLimits, DebugPauseHandle, DebugRecordingEnvelope,
    DebugRecordingEvent, DebugRunResult, DebugScope, DebugScopeKind, DebugSession,
    DebugSessionError, DebugSessionState, DebugStop, DebugStopReason,
    DebugStorageInitializationResult, DebugStringMutationResult, DebugTask, DebugTaskEvent,
    DebugTaskEventKind, DebugTaskResultReplacement, DebugTaskState, DebugTerminalEvent,
    DebugTerminalHandle, DebugTerminalKeyEvent, DebugTerminalKeyKind, DebugTerminalMouseAction,
    DebugTerminalMouseButton, DebugTermination, DebugUnaryOperation, DebugVariable,
    DebugVariantConstructionResult, DebugVariantDescription, DebugVariantField, DebugVariantInfo,
    DebuggeeChannelState, DebuggeeInputResult, FunctionBreakpoint, LiveImageClassification,
    LiveImageReplaceResult, LiveImageUpdateClass, MAX_RECORDING_EVENTS, Paginated,
    RECORDING_ENVELOPE_VERSION, SourceBreakpoint, SourceLocation,
};
pub use debug::{DebugSnapshot, DebugTaskSnapshot, non_executing};
pub use diagnostics::VmError;
pub(crate) use diagnostics::runtime_error;
pub use failure_capture::{
    FailureBinding, FailureDiagnostic, FailureFrame, FailureGlobal, FailureGlobals,
    FailureObservation, FailureScalar, FailureSnapshot, FailureTask, FailureTaskCatalog,
    FailureValue,
};

/// Captured console output produced by the virtual machine.
pub type VmOutput = fpas_std::CapturedOutput;

pub(crate) use shared::{TaskAnyPoll, TaskBatchPoll, TaskResultPoll, TaskResultState, TaskTimers};

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
    use_local_globals: bool,
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
        let use_local_globals = pool_size == 0 && globals::can_use_local_globals(&executable);
        Self {
            executable,
            has_run: false,
            globals,
            layouts,
            hosted: Arc::new(HostedState::new(console, arguments)),
            pool_size,
            use_local_globals,
            scheduler: Arc::new(TaskScheduler::new()),
        }
    }

    /// Queue one line for hosted `ReadText` and `ReadLn` calls.
    pub fn push_readln_input(&mut self, line: &str) {
        self.hosted
            .text_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push_line(line);
    }

    /// Signal EOF after pending characters and queued text lines have been read.
    pub fn close_text_input(&mut self) {
        self.hosted
            .text_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .close_input();
    }

    /// Sets the directory returned by `Std.Test.ScratchDir()` for this VM.
    pub fn set_test_scratch_dir(&mut self, path: std::path::PathBuf) {
        *self
            .hosted
            .test_scratch_dir
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = path;
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
            hosted: Arc::clone(&self.hosted),
        }
    }

    /// Authorize explicit `Std.Server` signal observation and process-wide escalation.
    ///
    /// Default VMs never acquire this authority. Standalone hosts may opt in before run;
    /// a program must still request signal observation or forced exit separately.
    /// See `docs/pascal/std/network/server.md`.
    pub fn allow_process_lifecycle(&mut self) {
        self.hosted
            .servers
            .process_authority
            .store(true, std::sync::atomic::Ordering::Release);
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
        let result = self.layouts.clone().and_then(|layouts| {
            Worker::for_function_with_state(
                Arc::clone(&self.executable),
                function,
                arguments,
                Arc::clone(&self.globals),
                layouts,
                Arc::clone(&self.hosted),
            )?
            .run()
        });
        self.hosted.close_process_handles();
        self.hosted.close_file_handles();
        self.hosted.wait_process_handles();
        result
    }
}

/// Cloneable cooperative-cancellation handle for a running [`Vm`].
#[derive(Clone)]
pub struct ShutdownHandle {
    scheduler: Arc<TaskScheduler>,
    hosted: Arc<HostedState>,
}

impl ShutdownHandle {
    /// Request cancellation and interrupt blocking hosted operations.
    pub fn shutdown(&self) {
        self.scheduler.request_cancel();
        self.hosted.shutdown_blocking_operations();
    }
}

pub use debug::{
    DEFAULT_TRACE_EVENTS, DebugTraceCaptureSource, DebugTraceEvent, DebugTraceIdentity,
    DebugTraceLocation, DebugTracePathStep, DebugTraceReport, DebugTraceWatch, MAX_TRACE_BYTES,
    MAX_TRACE_EVENTS,
};

pub use debug::{DebugScheduleReport, SCHEDULE_ALGORITHM};
