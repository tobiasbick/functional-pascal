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
mod domain;
mod execute;
mod failure_capture;
mod frame;
mod globals;
mod host_value;
mod hosted;
mod inspection;
mod layouts;
mod profile;
mod register_stack;
mod run;
mod shared;
mod tasks;
mod value_ops;
mod worker;

#[cfg(test)]
mod tests;

use std::sync::{Arc, RwLock};

use crate::{FunctionId, MemoryDomain, Value, VerifiedExecutable};

use self::host_value::HostValues;
use self::hosted::HostedState;
use self::layouts::RuntimeLayouts;
use self::tasks::TaskScheduler;
use self::worker::{FunctionInputs, Worker};

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
pub use host_value::{HostArgument, HostChild, HostEnumInfo, HostValue, HostValueError};
pub use profile::{
    DebugProfile, LineProfile, MAX_BATCH_STEPS, ProfileReport, ProfileSample, RoutineProfile,
};

/// Captured console output produced by the virtual machine.
pub type VmOutput = crate::stdlib::CapturedOutput;

pub(crate) use shared::{TaskAnyPoll, TaskBatchPoll, TaskResultPoll, TaskResultState, TaskTimers};

const TIMESLICE: u32 = 65536;
/// Most scheduling steps one dispatch batch executes.
const BATCH_STEPS: u32 = 256;

/// Successful root execution result and diagnostic instruction count.
#[derive(Debug, Clone)]
pub struct Execution {
    /// Root or callback return value.
    pub value: HostValue,
    /// Number of packed instructions dispatched by this run.
    pub instruction_count: u64,
}

/// Worker outcome before the public host boundary seals its value.
#[derive(Debug)]
pub(super) struct RuntimeExecution {
    value: Value,
    instruction_count: u64,
    _roots: Option<crate::RootValues>,
}
impl RuntimeExecution {
    fn rooted(value: Value, instruction_count: u64, domain: &crate::MemoryDomain) -> Self {
        let roots = domain.root_values(crate::RootKind::Native, vec![value.clone()]);
        Self {
            value,
            instruction_count,
            _roots: Some(roots),
        }
    }
}

/// Single-use VM for a pre-verified executable.
///
/// Code metadata may be shared while globals, hosted resources, and memory domains
/// belong to this execution session. Returned values cross the registered host boundary.
pub struct Vm {
    domain: MemoryDomain,
    host_values: HostValues,
    executable: Arc<VerifiedExecutable>,
    has_run: bool,
    session: Option<crate::DomainSession>,
    globals: Arc<RwLock<Vec<Option<Value>>>>,
    layouts: Result<Arc<RuntimeLayouts>, VmError>,
    hosted: Arc<HostedState>,
    pool_size: usize,
    use_local_globals: bool,
    scheduler: Arc<TaskScheduler>,
}

impl Vm {
    /// Select collection thresholds for this VM and every domain derived from it.
    /// Configure before starting execution; existing constructors select defaults.
    #[must_use]
    pub fn with_collection_schedule(self, schedule: crate::CollectionSchedule) -> Self {
        crate::execution::vm::domain::configure(&self.domain, schedule);
        self
    }

    /// Construct an isolated VM from an owned verified executable.
    #[must_use]
    pub fn new(executable: VerifiedExecutable) -> Self {
        Self::from_shared_with_host(
            Arc::new(executable),
            crate::stdlib::Console::new(),
            Vec::new(),
        )
    }

    /// Construct an isolated VM sharing immutable verified executable metadata.
    #[must_use]
    pub fn from_shared(executable: Arc<VerifiedExecutable>) -> Self {
        Self::from_shared_with_host(executable, crate::stdlib::Console::new(), Vec::new())
    }

    /// Construct an isolated VM with process arguments visible through `Std.Os.Args`.
    #[must_use]
    pub fn with_args(executable: VerifiedExecutable, arguments: Vec<String>) -> Self {
        Self::from_shared_with_host(
            Arc::new(executable),
            crate::stdlib::Console::new(),
            arguments,
        )
    }

    /// Construct an isolated VM that streams hosted console output.
    #[must_use]
    pub fn with_writer(
        executable: VerifiedExecutable,
        writer: Box<dyn std::io::Write + Send>,
    ) -> Self {
        Self::from_shared_with_host(
            Arc::new(executable),
            crate::stdlib::Console::with_writer(writer),
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
            crate::stdlib::Console::with_writer(writer),
            arguments,
        )
    }

    fn from_shared_with_host(
        executable: Arc<VerifiedExecutable>,
        console: crate::stdlib::Console,
        arguments: Vec<String>,
    ) -> Self {
        let globals = Arc::new(RwLock::new(vec![
            None;
            executable.executable().globals.len()
        ]));
        let layouts =
            RuntimeLayouts::build(executable.executable(), crate::InstructionAddress::new(0))
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
        let (domain, session) = crate::execution::vm::domain::new_session_domain();
        let hosted = Arc::new(HostedState::new(console, arguments));
        let scheduler = Arc::new(TaskScheduler::new());
        // Synchronous Vm::call workers need no scheduler, but their shutdown handle still
        // owns the session boundary and must close unused cases while a call is blocked.
        scheduler.attach_cases(&hosted.cases);
        Self {
            host_values: HostValues::for_executable(
                domain.clone(),
                &executable,
                Arc::clone(&globals),
            ),
            domain,
            executable,
            has_run: false,
            session: Some(session),
            globals,
            layouts,
            hosted,
            pool_size,
            use_local_globals,
            scheduler,
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
    pub fn push_key_event(&mut self, event: crate::stdlib::ConsoleKeyEvent) {
        self.hosted
            .key_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push_key_event(event);
    }

    /// Queue one unified hosted console event.
    pub fn push_console_event(&mut self, event: crate::stdlib::ConsoleEvent) {
        self.hosted
            .key_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push_console_event(event);
    }

    /// Queue terminal-host input and update console dimensions for resize events.
    pub fn push_terminal_event(&mut self, event: &DebugTerminalEvent) {
        let mut console = self
            .hosted
            .console
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut input = self
            .hosted
            .key_input
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        hosted::terminal_input::queue_event(&mut console, &mut input, event);
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
    pub fn screen_snapshot(&self) -> crate::stdlib::ScreenSnapshot {
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

    /// Authorize explicit `Std.Net.Server` signal observation and process-wide escalation.
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
        arguments: Vec<HostArgument>,
    ) -> Result<Execution, VmError> {
        if self.scheduler.is_shutdown() || std::mem::replace(&mut self.has_run, true) {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                crate::InstructionAddress::new(0),
                fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN,
                "This VM instance has already been run",
                "Register VM instances are single-use. Construct a new instance for each callback.",
            ));
        }
        let result = self.layouts.clone().and_then(|layouts| {
            let (arguments, argument_roots) =
                self.host_values.admit(arguments, &self.executable)?;
            let worker = Worker::for_function_in_domain(
                Arc::clone(&self.executable),
                function,
                FunctionInputs {
                    arguments: &arguments,
                    captures: &[],
                    domain: self.domain.clone(),
                },
                Arc::clone(&self.globals),
                layouts,
                Arc::clone(&self.hosted),
            );
            crate::release_values(arguments);
            drop(argument_roots);
            worker?.run()
        });
        self.hosted.close_process_handles();
        self.hosted.close_file_handles();
        self.hosted.wait_process_handles();
        self.hosted.cases.close_all();
        let result = result.map(|execution| self.host_values.export(execution));
        self.session.take();
        result
    }
}

impl Drop for Vm {
    fn drop(&mut self) {
        self.hosted.cases.close_all();
        self.host_values.retire_globals(&self.globals);
        self.session.take();
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
