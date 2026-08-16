//! Register-window state and execution loop.

use std::cell::{Cell, RefCell};
use std::sync::{Arc, RwLock};

use fpas_bytecode::{FunctionId, InstructionAddress, Value, VerifiedExecutable};

use super::debug::initializer_suppression::SourceInitializerTarget;
use super::dispatch::DispatchStep;
use super::frame::CallFrame;
use super::hosted::HostedState;
use super::layouts::RuntimeLayouts;
use super::tasks::{DebugClock, TaskScheduler, TaskState, TaskSuspension, TaskSuspensionState};
use super::{Execution, VmError, diagnostics};

pub(super) struct Worker {
    pub executable: Arc<VerifiedExecutable>,
    pub function: FunctionId,
    pub ip: usize,
    pub base: usize,
    pub registers: Vec<Value>,
    // Physical register storage retains a high-water mark; this is the live prefix.
    pub(super) active_register_count: usize,
    pub(super) register_initialized: Vec<bool>,
    pub globals: Arc<RwLock<Vec<Option<Value>>>>,
    pub layouts: Arc<RuntimeLayouts>,
    pub hosted: Arc<HostedState>,
    pub call_stack: Vec<CallFrame>,
    pub instruction_count: u64,
    pub(super) callback_instruction_count: Cell<u64>,
    pub(super) callback_worker: RefCell<Option<Box<Worker>>>,
    pub current_address: InstructionAddress,
    pub scheduler: Option<Arc<TaskScheduler>>,
    pub task_id: u64,
    pub retain_result: bool,
    pub instructions_until_yield: u32,
    pub suspend_requested: bool,
    pub(in crate::vm) debug_tasks: bool,
    pub(in crate::vm) task_suspension: Option<TaskSuspension>,
    pub(in crate::vm) debug_clock: Option<Arc<DebugClock>>,
    pub(in crate::vm) suppressed_initializers: Vec<SourceInitializerTarget>,
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
        Ok(Self {
            executable,
            function: entry,
            ip,
            base: 0,
            registers,
            active_register_count: usize::from(register_count),
            register_initialized,
            globals,
            layouts,
            hosted,
            call_stack: Vec::new(),
            instruction_count: 0,
            callback_instruction_count: Cell::new(0),
            callback_worker: RefCell::new(None),
            current_address: start,
            scheduler: None,
            task_id: 0,
            retain_result: false,
            instructions_until_yield: super::TIMESLICE,
            suspend_requested: false,
            debug_tasks: false,
            task_suspension: None,
            debug_clock: None,
            suppressed_initializers: Vec::new(),
        })
    }

    pub(super) fn with_scheduler(mut self, scheduler: Option<Arc<TaskScheduler>>) -> Self {
        self.scheduler = scheduler;
        self
    }

    /// Enable cooperative task suspension for debugger-owned execution.
    pub(in crate::vm) fn with_debug_tasks(mut self, clock: Arc<DebugClock>) -> Self {
        self.debug_tasks = true;
        self.debug_clock = Some(clock);
        self
    }

    pub(super) fn pool_template(&self) -> Self {
        Self {
            executable: Arc::clone(&self.executable),
            function: self.function,
            ip: self.ip,
            base: 0,
            registers: Vec::new(),
            active_register_count: 0,
            register_initialized: Vec::new(),
            globals: Arc::clone(&self.globals),
            layouts: Arc::clone(&self.layouts),
            hosted: Arc::clone(&self.hosted),
            call_stack: Vec::new(),
            instruction_count: 0,
            callback_instruction_count: Cell::new(0),
            callback_worker: RefCell::new(None),
            current_address: self.current_address,
            scheduler: self.scheduler.clone(),
            task_id: 0,
            retain_result: false,
            instructions_until_yield: super::TIMESLICE,
            suspend_requested: false,
            debug_tasks: self.debug_tasks,
            task_suspension: None,
            debug_clock: self.debug_clock.clone(),
            suppressed_initializers: Vec::new(),
        }
    }

    pub(super) fn worker_for_task(&self, task: TaskState) -> Self {
        let active_register_count = task.registers.len();
        debug_assert_eq!(
            task.register_initialized.len(),
            active_register_count,
            "task register initialization bits must match saved register values"
        );
        Self {
            executable: Arc::clone(&self.executable),
            function: task.function,
            ip: task.ip,
            base: task.base,
            registers: task.registers,
            active_register_count,
            register_initialized: task.register_initialized,
            globals: Arc::clone(&self.globals),
            layouts: Arc::clone(&self.layouts),
            hosted: Arc::clone(&self.hosted),
            call_stack: task.frames,
            instruction_count: task.instruction_count,
            callback_instruction_count: Cell::new(0),
            callback_worker: RefCell::new(None),
            current_address: self.current_address,
            scheduler: self.scheduler.clone(),
            task_id: task.id,
            retain_result: task.retain_result,
            instructions_until_yield: super::TIMESLICE,
            suspend_requested: false,
            debug_tasks: self.debug_tasks,
            task_suspension: None,
            debug_clock: self.debug_clock.clone(),
            suppressed_initializers: task.suppressed_initializers,
        }
    }

    pub(super) fn take_task_state(&mut self) -> TaskState {
        self.registers.truncate(self.active_register_count);
        self.register_initialized
            .truncate(self.active_register_count);
        TaskState {
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
        }
    }

    pub(super) fn suspend_and_enqueue(&mut self) {
        if let Some(scheduler) = self.scheduler.clone() {
            let state = self.take_task_state();
            scheduler.enqueue(state);
            self.suspend_requested = true;
        }
    }

    /// Return the current cooperative suspension kind, when this task is blocked.
    pub(in crate::vm) fn debug_suspension_state(&self) -> Option<TaskSuspensionState> {
        self.task_suspension.as_ref().map(|suspension| {
            let Some(clock) = self.debug_clock.as_deref() else {
                unreachable!("debug task suspension requires a debugger clock")
            };
            suspension.state(clock)
        })
    }

    pub(super) fn run_task(&mut self) -> Result<Option<Value>, VmError> {
        loop {
            if self
                .scheduler
                .as_ref()
                .is_some_and(|scheduler| scheduler.is_aborted())
            {
                return Ok(None);
            }
            match self.dispatch_one()? {
                DispatchStep::Continue => {}
                DispatchStep::Suspend => return Ok(None),
                DispatchStep::Return(value) => return Ok(Some(value)),
            }
            if self.task_id != 0 {
                self.instructions_until_yield = self.instructions_until_yield.saturating_sub(1);
                if self.instructions_until_yield == 0 {
                    self.suspend_and_enqueue();
                    return Ok(None);
                }
            }
        }
    }

    pub fn run(mut self) -> Result<Execution, VmError> {
        self.run_in_place()
    }

    pub(super) fn run_in_place(&mut self) -> Result<Execution, VmError> {
        loop {
            if self
                .scheduler
                .as_ref()
                .is_some_and(|scheduler| scheduler.is_aborted())
            {
                return Err(diagnostics::at_address(
                    self.executable.executable(),
                    self.current_address,
                    fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN,
                    "Register VM execution was canceled",
                    "Create a new VM instance to run the program again.",
                ));
            }
            match self.dispatch_one()? {
                DispatchStep::Continue => {}
                DispatchStep::Suspend => {
                    return Err(diagnostics::internal(
                        self.executable.executable(),
                        self.current_address,
                        "Root register execution suspended unexpectedly",
                    ));
                }
                DispatchStep::Return(value) => {
                    return Ok(Execution {
                        value,
                        instruction_count: self
                            .instruction_count
                            .saturating_add(self.callback_instruction_count.get()),
                    });
                }
            }
        }
    }

    pub fn unavailable_opcode(&self, opcode: fpas_bytecode::Opcode) -> VmError {
        diagnostics::internal(
            self.executable.executable(),
            self.current_address,
            format!("Opcode {opcode:?} is reserved and cannot be executed"),
        )
    }
}
