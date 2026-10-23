//! Register-window state and execution loop.

mod run;

#[cfg(test)]
mod domains;

use std::cell::{Cell, RefCell};
use std::sync::{Arc, RwLock};

use fpas_bytecode::{FunctionId, InstructionAddress, MemoryDomain, Value, VerifiedExecutable};

use super::debug::initializer_suppression::SourceInitializerTarget;
use super::frame::CallFrame;
use super::hosted::HostedState;
use super::hosted::callbacks::CallbackContinuation;
use super::layouts::RuntimeLayouts;
use super::tasks::{TaskClock, TaskScheduler, TaskState, TaskSuspension, TaskSuspensionState};
use super::{VmError, diagnostics};

/// Inputs already assigned to the logical task that will execute them.
pub(super) struct FunctionInputs<'a> {
    pub arguments: &'a [Value],
    pub captures: &'a [Value],
    pub domain: MemoryDomain,
}

pub(super) struct Worker {
    /// Stable cell ownership across worker migration and synchronous callbacks.
    pub domain: MemoryDomain,
    /// Optional ownership record transferred with this task, absent without D1.
    pub(in crate::vm) capture_record: Option<Arc<super::failure_capture::catalog::TaskRecord>>,
    /// Whether this worker owns the top-level root, rather than a synchronous callback.
    pub(in crate::vm) capture_root: bool,
    /// A synchronous callback has FPAS callers outside this worker's register windows.
    pub(in crate::vm) capture_callback: bool,
    pub(in crate::vm) supervision: Option<Box<super::tasks::supervision::SupervisedTask>>,
    pub executable: Arc<VerifiedExecutable>,
    pub function: FunctionId,
    pub ip: usize,
    pub base: usize,
    // Live registers of all active frames; the length is the active window (see register_stack).
    pub registers: Vec<Value>,
    pub(super) register_initialized: Vec<bool>,
    pub globals: Arc<RwLock<Vec<Option<Value>>>>,
    pub(super) local_globals: Option<Vec<Option<Value>>>,
    pub layouts: Arc<RuntimeLayouts>,
    pub hosted: Arc<HostedState>,
    pub call_stack: Vec<CallFrame>,
    pub instruction_count: u64,
    pub(super) callback_instruction_count: Cell<u64>,
    pub(super) callback_worker: RefCell<Option<Box<Worker>>>,
    /// Whether the cached synchronous callback propagated the current failure.
    pub(super) callback_failed: Cell<bool>,
    pub(super) callback_continuations: Vec<CallbackContinuation>,
    pub current_address: InstructionAddress,
    pub scheduler: Option<Arc<TaskScheduler>>,
    pub task_id: u64,
    pub retain_result: bool,
    pub instructions_until_yield: u32,
    pub suspend_requested: bool,
    pub(in crate::vm) debug_tasks: bool,
    pub(in crate::vm) debug_schedule: Option<Arc<super::debug::scheduling::ScheduleGuard>>,
    pub(in crate::vm) debug_local_trace: Option<Box<super::debug::trace::LocalTraceState>>,
    pub(in crate::vm) debug_last_global_store: Option<u32>,
    pub(in crate::vm) debug_trace: Option<Arc<std::sync::Mutex<super::debug::trace::TraceHistory>>>,
    pub(in crate::vm) task_suspension: Option<TaskSuspension>,
    pub(in crate::vm) pending_completion: Option<Result<Value, VmError>>,
    pub(in crate::vm) task_clock: Option<Arc<TaskClock>>,
    pub(in crate::vm) suppressed_initializers: Vec<SourceInitializerTarget>,
    /// Task scopes opened by this task's call frames, innermost last.
    pub(in crate::vm) task_scopes: Vec<super::tasks::scopes::OpenScope>,
    /// Shared control of the task scope that owns this task, if any.
    pub(in crate::vm) owner_scope: Option<Arc<super::tasks::scopes::ScopeControl>>,
    /// Cancellation controls inherited by a synchronous callback from its caller.
    pub(in crate::vm) enclosing_scope_controls: Vec<Arc<super::tasks::scopes::ScopeControl>>,
    /// Observation identity of the failure propagated by a scope exit or task wait.
    pub(in crate::vm) scope_failure_sequence: Cell<Option<u64>>,
    /// Child identity whose failure is already carried by the body's terminal error.
    pub(in crate::vm) observed_task_failure: Cell<Option<u64>>,
    pub(in crate::vm) execution: Option<fpas_bytecode::ExecutionRoots>,
    /// Token states already validated by this worker; polling them needs no registry lock.
    pub(in crate::vm) cancellation_views: RefCell<Vec<super::cancellation::CancellationView>>,
    /// Local profile samples, created on the first batch of a profiled run.
    pub(in crate::vm) profile: Option<Box<super::profile::ProfileBuffer>>,
}

impl Worker {
    pub(crate) fn record_value(
        &self,
        type_name: &str,
        values: Vec<Value>,
        location: fpas_bytecode::SourceLocation,
    ) -> Result<Value, VmError> {
        fpas_std::AggregateFactory::record(self.layouts.as_ref(), type_name, values, location)
    }

    #[cfg(test)]
    pub fn new(executable: Arc<VerifiedExecutable>) -> Result<Self, VmError> {
        let entry = executable.executable().entry;
        let globals = Arc::new(RwLock::new(vec![
            None;
            executable.executable().globals.len()
        ]));
        let layouts = Arc::new(RuntimeLayouts::build(
            executable.executable(),
            InstructionAddress::new(0),
        )?);
        let hosted = Arc::new(HostedState::new(fpas_std::Console::new(), Vec::new()));
        Self::for_function_with_state(executable, entry, Vec::new(), globals, layouts, hosted)
    }

    #[cfg(test)]
    /// Construct a test worker with worker-owned global slots.
    pub fn new_with_local_globals(executable: Arc<VerifiedExecutable>) -> Result<Self, VmError> {
        Self::new(executable).map(|worker| worker.with_local_globals(true))
    }

    pub fn for_function_with_state(
        executable: Arc<VerifiedExecutable>,
        entry: FunctionId,
        arguments: Vec<Value>,
        globals: Arc<RwLock<Vec<Option<Value>>>>,
        layouts: Arc<RuntimeLayouts>,
        hosted: Arc<HostedState>,
    ) -> Result<Self, VmError> {
        Self::for_function_with_captures(
            executable,
            entry,
            &arguments,
            &[],
            globals,
            layouts,
            hosted,
        )
    }

    pub(super) fn for_function_with_captures(
        executable: Arc<VerifiedExecutable>,
        entry: FunctionId,
        arguments: &[Value],
        captures: &[Value],
        globals: Arc<RwLock<Vec<Option<Value>>>>,
        layouts: Arc<RuntimeLayouts>,
        hosted: Arc<HostedState>,
    ) -> Result<Self, VmError> {
        Self::for_function_in_domain(
            executable,
            entry,
            FunctionInputs {
                arguments,
                captures,
                domain: crate::vm::domain::new_domain(),
            },
            globals,
            layouts,
            hosted,
        )
    }

    /// Construct an invocation in its caller's or reusable session's domain.
    pub(super) fn for_function_in_domain(
        executable: Arc<VerifiedExecutable>,
        entry: FunctionId,
        inputs: FunctionInputs<'_>,
        globals: Arc<RwLock<Vec<Option<Value>>>>,
        layouts: Arc<RuntimeLayouts>,
        hosted: Arc<HostedState>,
    ) -> Result<Self, VmError> {
        let FunctionInputs {
            arguments,
            captures,
            domain,
        } = inputs;
        let image = executable.executable();
        let info = image
            .functions
            .get(usize::from(entry.get()))
            .ok_or_else(|| {
                diagnostics::internal(
                    image,
                    InstructionAddress::new(0),
                    "Root function metadata is missing",
                )
            })?;
        let start = info.code.start;
        if arguments.len() != usize::from(info.arity)
            || captures.len() != usize::from(info.capture_count)
        {
            return Err(diagnostics::internal(
                image,
                start,
                format!(
                    "Callback entry signature mismatch: expected {} arguments and {} captures, got {} arguments and {} captures",
                    info.arity,
                    info.capture_count,
                    arguments.len(),
                    captures.len()
                ),
            ));
        }
        let register_count = info.register_count;
        let ip = usize::try_from(start.get()).map_err(|_| {
            diagnostics::internal(
                image,
                start,
                "Root instruction address does not fit this host",
            )
        })?;
        let (registers, register_initialized) = Self::register_window(
            usize::from(register_count),
            arguments.iter().chain(captures).cloned(),
        );
        let execution = domain.execution_roots().map_err(|error| {
            diagnostics::at_address(
                image,
                start,
                fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN,
                error.to_string(),
                "Create a live memory domain before executing FPAS.",
            )
        })?;
        let worker = Self {
            execution: Some(execution),
            domain,
            capture_record: None,
            capture_root: false,
            capture_callback: false,
            supervision: None,
            executable,
            function: entry,
            ip,
            base: 0,
            registers,
            register_initialized,
            globals,
            local_globals: None,
            layouts,
            hosted,
            call_stack: Vec::new(),
            instruction_count: 0,
            callback_instruction_count: Cell::new(0),
            callback_worker: RefCell::new(None),
            callback_failed: Cell::new(false),
            callback_continuations: Vec::new(),
            current_address: start,
            scheduler: None,
            task_id: 0,
            retain_result: false,
            instructions_until_yield: super::TIMESLICE,
            cancellation_views: RefCell::new(Vec::new()),
            profile: None,
            suspend_requested: false,
            debug_tasks: false,
            debug_schedule: None,
            debug_local_trace: None,
            debug_last_global_store: None,
            debug_trace: None,
            task_suspension: None,
            pending_completion: None,
            task_clock: Some(Arc::new(TaskClock::realtime())),
            suppressed_initializers: Vec::new(),
            task_scopes: Vec::new(),
            owner_scope: None,
            enclosing_scope_controls: Vec::new(),
            scope_failure_sequence: Cell::new(None),
            observed_task_failure: Cell::new(None),
        };
        worker.publish_execution(&[])?;
        Ok(worker)
    }

    pub(super) fn with_scheduler(mut self, scheduler: Option<Arc<TaskScheduler>>) -> Self {
        if let Some(scheduler) = &scheduler {
            scheduler.attach_cases(&self.hosted.cases);
        }
        self.scheduler = scheduler;
        self
    }

    /// Enable cooperative task suspension for debugger-owned execution.
    pub(in crate::vm) fn with_debug_tasks(mut self, clock: Arc<TaskClock>) -> Self {
        self.debug_tasks = true;
        self.task_clock = Some(clock);
        self
    }

    /// Record a successful global store for debugger data breakpoints.
    pub(in crate::vm) fn note_debug_global_store(&mut self, index: usize) {
        if self.debug_tasks {
            self.debug_last_global_store = u32::try_from(index).ok();
        }
    }

    /// Take the last debug-owned global store, if one is pending.
    pub(in crate::vm) fn take_debug_global_store(&mut self) -> Option<u32> {
        self.debug_last_global_store.take()
    }

    /// Share immutable execution context without transferring task-owned registers.
    pub(super) fn pool_template(&self) -> Self {
        Self {
            domain: crate::vm::domain::related_domain(&self.domain),
            execution: None,
            supervision: None,
            capture_record: None,
            capture_root: false,
            capture_callback: false,
            executable: Arc::clone(&self.executable),
            function: self.function,
            ip: self.ip,
            base: 0,
            registers: Vec::new(),
            register_initialized: Vec::new(),
            globals: Arc::clone(&self.globals),
            local_globals: None,
            layouts: Arc::new(self.layouts.detached()),
            hosted: Arc::clone(&self.hosted),
            call_stack: Vec::new(),
            instruction_count: 0,
            callback_instruction_count: Cell::new(0),
            callback_worker: RefCell::new(None),
            callback_failed: Cell::new(false),
            callback_continuations: Vec::new(),
            current_address: self.current_address,
            scheduler: self.scheduler.clone(),
            task_id: 0,
            retain_result: false,
            instructions_until_yield: super::TIMESLICE,
            cancellation_views: RefCell::new(Vec::new()),
            profile: None,
            suspend_requested: false,
            debug_tasks: self.debug_tasks,
            debug_schedule: self.debug_schedule.clone(),
            debug_local_trace: self
                .debug_local_trace
                .as_ref()
                .map(|state| Box::new(state.for_task())),
            debug_last_global_store: None,
            debug_trace: None,
            task_suspension: None,
            pending_completion: None,
            task_clock: self.task_clock.clone(),
            suppressed_initializers: Vec::new(),
            task_scopes: Vec::new(),
            owner_scope: None,
            enclosing_scope_controls: Vec::new(),
            scope_failure_sequence: Cell::new(None),
            observed_task_failure: Cell::new(None),
        }
    }

    /// Take a saved task's execution and optional inspection ownership.
    pub(super) fn worker_for_task(&self, mut task: TaskState) -> Self {
        if task.execution.is_none() {
            task.execution = task.domain.execution_roots().ok();
        }
        debug_assert_eq!(
            task.register_initialized.len(),
            task.registers.len(),
            "task register initialization bits must match saved register values"
        );
        Self {
            execution: task.execution,
            domain: task.domain,
            supervision: task.supervision,
            capture_record: task.capture_record,
            capture_root: false,
            capture_callback: false,
            executable: Arc::clone(&self.executable),
            function: task.function,
            ip: task.ip,
            base: task.base,
            registers: task.registers,
            register_initialized: task.register_initialized,
            globals: Arc::clone(&self.globals),
            local_globals: None,
            layouts: Arc::clone(&self.layouts),
            hosted: Arc::clone(&self.hosted),
            call_stack: task.frames,
            instruction_count: task.instruction_count,
            callback_instruction_count: Cell::new(0),
            callback_worker: RefCell::new(None),
            callback_failed: Cell::new(false),
            callback_continuations: task.callback_continuations,
            current_address: self.current_address,
            scheduler: self.scheduler.clone(),
            task_id: task.id,
            retain_result: task.retain_result,
            instructions_until_yield: super::TIMESLICE,
            cancellation_views: RefCell::new(Vec::new()),
            profile: None,
            suspend_requested: false,
            debug_tasks: self.debug_tasks,
            debug_schedule: self.debug_schedule.clone(),
            debug_local_trace: self
                .debug_local_trace
                .as_ref()
                .map(|state| Box::new(state.for_task())),
            debug_last_global_store: None,
            debug_trace: self.debug_trace.clone(),
            task_suspension: task.suspension,
            pending_completion: task.pending_completion,
            task_clock: self.task_clock.clone(),
            suppressed_initializers: task.suppressed_initializers,
            task_scopes: task.task_scopes,
            owner_scope: task.owner_scope,
            enclosing_scope_controls: Vec::new(),
            scope_failure_sequence: Cell::new(None),
            observed_task_failure: Cell::new(task.observed_task_failure),
        }
    }

    /// Move all resumable task state to its next scheduler owner.
    pub(super) fn take_task_state(&mut self) -> TaskState {
        let state = TaskState {
            execution: self.execution.take(),
            domain: self.domain.clone(),
            capture_record: self.capture_record.take(),
            observed_task_failure: self.observed_task_failure.take(),
            pending_completion: self.pending_completion.take(),
            suspension: self.task_suspension.take(),
            supervision: self.supervision.take(),
            id: self.task_id,
            function: self.function,
            ip: self.ip,
            base: self.base,
            registers: std::mem::take(&mut self.registers),
            register_initialized: std::mem::take(&mut self.register_initialized),
            frames: std::mem::take(&mut self.call_stack),
            retain_result: self.retain_result,
            instruction_count: self.instruction_count,
            suppressed_initializers: std::mem::take(&mut self.suppressed_initializers),
            callback_continuations: std::mem::take(&mut self.callback_continuations),
            task_scopes: std::mem::take(&mut self.task_scopes),
            owner_scope: self.owner_scope.take(),
        };
        state.publish_execution();
        state
    }

    pub(super) fn suspend_and_enqueue(&mut self) {
        if let Some(scheduler) = self.scheduler.clone() {
            fpas_bytecode::profile::add(fpas_bytecode::profile::Counter::TaskYields, 1);
            let state = self.take_task_state();
            scheduler.enqueue(state);
            self.suspend_requested = true;
        }
    }

    /// Return the current cooperative suspension kind, when this task is blocked.
    pub(in crate::vm) fn debug_suspension_state(&self) -> Option<TaskSuspensionState> {
        self.task_suspension.as_ref().map(|suspension| {
            let Some(clock) = self.task_clock.as_deref() else {
                unreachable!("debug task suspension requires a debugger clock")
            };
            suspension.state(clock)
        })
    }

    pub fn unavailable_opcode(&self, opcode: fpas_bytecode::Opcode) -> VmError {
        diagnostics::internal(
            self.executable.executable(),
            self.current_address,
            format!("Opcode {opcode:?} is reserved and cannot be executed"),
        )
    }
}
