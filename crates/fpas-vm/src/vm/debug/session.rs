//! Debug session construction, state transitions, and controlled dispatch.

use fpas_bytecode::{InstructionAddress, VerifiedExecutable};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use super::breakpoints::{self, BoundBreakpoint, SourceBreakpoint};
use super::evaluation::{DebugEvaluateResult, DebugEvaluationLimits, DebugExpression};
use super::inspection::{
    DebugFrame, DebugInspectionLimits, DebugScope, DebugVariable, InspectionSnapshot, Paginated,
};
use super::types::{
    DebugErrorKind, DebugExecutionLimits, DebugRunResult, DebugSessionError, DebugSessionState,
    DebugStop, DebugStopReason, DebugTermination,
};
use crate::vm::hosted::HostedState;
use crate::vm::layouts::RuntimeLayouts;
use crate::vm::worker::Worker;

mod execution;

/// Thread-safe cooperative pause request handle.
#[derive(Clone)]
pub struct DebugPauseHandle {
    requested: Arc<AtomicBool>,
}

impl DebugPauseHandle {
    /// Request a pause at the next source sequence point after an instruction boundary.
    pub fn request_pause(&self) {
        self.requested.store(true, Ordering::Release);
    }
}

/// Single-use controlled execution session for one verified executable.
pub struct DebugSession {
    executable: Arc<VerifiedExecutable>,
    worker: Worker,
    state: DebugSessionState,
    breakpoints: Vec<BoundBreakpoint>,
    next_breakpoint_id: u64,
    pause_requested: Arc<AtomicBool>,
    last_stop: DebugStop,
    inspection_generation: u32,
    inspection: InspectionSnapshot,
    inspection_limits: DebugInspectionLimits,
    execution_limits: DebugExecutionLimits,
}

impl DebugSession {
    /// Construct a stopped debug session at the program entry.
    ///
    /// # Errors
    ///
    /// Returns an actionable error when the executable uses task spawning or runtime state cannot
    /// be initialized.
    pub fn new(executable: VerifiedExecutable) -> Result<Self, DebugSessionError> {
        Self::with_limits(
            executable,
            Vec::new(),
            DebugInspectionLimits::default(),
            DebugExecutionLimits::default(),
        )
    }

    /// Construct a stopped debug session with process arguments.
    ///
    /// # Errors
    ///
    /// Returns an actionable error when the executable uses task spawning or runtime state cannot
    /// be initialized.
    pub fn with_args(
        executable: VerifiedExecutable,
        arguments: Vec<String>,
    ) -> Result<Self, DebugSessionError> {
        Self::with_limits(
            executable,
            arguments,
            DebugInspectionLimits::default(),
            DebugExecutionLimits::default(),
        )
    }

    /// Construct a stopped session with process arguments and explicit inspection bounds.
    ///
    /// # Errors
    ///
    /// Returns an actionable error for unsupported task execution or invalid runtime state.
    pub fn with_args_and_limits(
        executable: VerifiedExecutable,
        arguments: Vec<String>,
        inspection_limits: DebugInspectionLimits,
    ) -> Result<Self, DebugSessionError> {
        Self::with_limits(
            executable,
            arguments,
            inspection_limits,
            DebugExecutionLimits::default(),
        )
    }

    /// Construct a stopped session with explicit inspection and execution bounds.
    ///
    /// # Errors
    ///
    /// Returns an actionable error for unsupported task execution or invalid runtime state.
    pub fn with_limits(
        executable: VerifiedExecutable,
        arguments: Vec<String>,
        inspection_limits: DebugInspectionLimits,
        execution_limits: DebugExecutionLimits,
    ) -> Result<Self, DebugSessionError> {
        let executable = Arc::new(executable);
        if let Some(function_id) = super::tasks::first_reachable_spawner(&executable) {
            let function = &executable.executable().functions[usize::from(function_id.get())];
            let name = executable
                .executable()
                .strings
                .get(function.name)
                .unwrap_or("<unknown>");
            return Err(DebugSessionError {
                kind: DebugErrorKind::UnsupportedTasks,
                message: format!(
                    "cannot debug executable because reachable function `{name}` can spawn tasks"
                ),
                hint: "Remove task spawning for this debug run; debugger task threads are intentionally deferred from V1."
                    .to_string(),
            });
        }
        let globals = Arc::new(RwLock::new(vec![
            None;
            executable.executable().globals.len()
        ]));
        let layouts = RuntimeLayouts::build(executable.executable(), InstructionAddress::new(0))
            .map(Arc::new)
            .map_err(runtime_initialization_error)?;
        let hosted = Arc::new(HostedState::new(fpas_std::Console::new(), arguments));
        let worker = Worker::for_function_with_state(
            Arc::clone(&executable),
            executable.executable().entry,
            Vec::new(),
            globals,
            layouts,
            hosted,
        )
        .map_err(runtime_initialization_error)?;
        let pause_requested = Arc::new(AtomicBool::new(false));
        let last_stop = stop_at_worker(
            &executable,
            &worker,
            DebugStopReason::Entry,
            Vec::new(),
            None,
        );
        let inspection_generation = 1;
        let inspection =
            InspectionSnapshot::capture(&worker, inspection_generation, inspection_limits);
        Ok(Self {
            executable,
            worker,
            state: DebugSessionState::Stopped,
            breakpoints: Vec::new(),
            next_breakpoint_id: 1,
            pause_requested,
            last_stop,
            inspection_generation,
            inspection,
            inspection_limits,
            execution_limits,
        })
    }

    /// Return the current session state.
    #[must_use]
    pub const fn state(&self) -> DebugSessionState {
        self.state
    }

    /// Return the latest stable stop snapshot.
    #[must_use]
    pub const fn last_stop(&self) -> &DebugStop {
        &self.last_stop
    }

    /// Return captured program output accumulated by the session.
    #[must_use]
    pub fn output(&self) -> super::super::VmOutput {
        self.worker
            .hosted
            .console
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .output()
            .clone()
    }

    /// Create a handle that can request cooperative pause while a resume operation runs.
    #[must_use]
    pub fn pause_handle(&self) -> DebugPauseHandle {
        DebugPauseHandle {
            requested: Arc::clone(&self.pause_requested),
        }
    }

    /// Return a bounded page of logical frames for the current stop.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or inspection-limit error.
    pub fn stack(
        &self,
        start: usize,
        count: usize,
    ) -> Result<Paginated<DebugFrame>, DebugSessionError> {
        self.require_inspectable("stack")?;
        self.inspection.stack(start, count)
    }

    /// Return source scopes for one frame in the current stop snapshot.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or expired-frame error.
    pub fn scopes(&self, frame_id: u64) -> Result<Vec<DebugScope>, DebugSessionError> {
        self.require_inspectable("scopes")?;
        self.inspection.scopes(frame_id)
    }

    /// Return one bounded page of variables or aggregate children.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state, expired-reference, or inspection-limit error.
    pub fn variables(
        &mut self,
        reference: u64,
        start: usize,
        count: usize,
    ) -> Result<Paginated<DebugVariable>, DebugSessionError> {
        self.require_inspectable("variables")?;
        self.inspection.variables(reference, start, count)
    }

    /// Evaluate one validated read-only expression against the current stop snapshot.
    ///
    /// A missing frame selects globals only. Supplied frame identifiers and returned aggregate
    /// handles are valid only for the current stop.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state, frame, name, type, domain, unavailable-value, or limit error.
    pub fn evaluate(
        &mut self,
        expression: &DebugExpression,
        frame_id: Option<u64>,
    ) -> Result<DebugEvaluateResult, DebugSessionError> {
        self.evaluate_with_limits(expression, frame_id, DebugEvaluationLimits::default())
    }

    /// Evaluate one validated read-only expression with explicit resource limits.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state, frame, name, type, domain, unavailable-value, or limit error.
    pub fn evaluate_with_limits(
        &mut self,
        expression: &DebugExpression,
        frame_id: Option<u64>,
        limits: DebugEvaluationLimits,
    ) -> Result<DebugEvaluateResult, DebugSessionError> {
        self.require_inspectable("evaluate")?;
        self.inspection.evaluate(expression, frame_id, limits)
    }

    /// Evaluate one validated breakpoint condition as a strict Boolean value.
    ///
    /// # Errors
    ///
    /// Returns the normal evaluation failures or a type error for a non-Boolean result.
    pub fn evaluate_boolean(
        &self,
        expression: &DebugExpression,
        frame_id: Option<u64>,
    ) -> Result<bool, DebugSessionError> {
        self.require_inspectable("evaluate.condition")?;
        self.inspection
            .evaluate_boolean(expression, frame_id, DebugEvaluationLimits::default())
    }

    /// Add one source breakpoint and return its verified or unverified binding.
    ///
    /// # Errors
    ///
    /// Returns [`DebugErrorKind::InvalidState`] after termination or runtime failure.
    pub fn set_breakpoint(
        &mut self,
        requested: SourceBreakpoint,
    ) -> Result<BoundBreakpoint, DebugSessionError> {
        self.require_stopped("breakpoint.set")?;
        let id = self.next_breakpoint_id;
        self.next_breakpoint_id = self.next_breakpoint_id.saturating_add(1);
        let breakpoint = breakpoints::bind(&self.executable, id, requested);
        self.breakpoints.push(breakpoint.clone());
        Ok(breakpoint)
    }

    /// Remove one session breakpoint.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or unknown-breakpoint error.
    pub fn clear_breakpoint(&mut self, id: u64) -> Result<(), DebugSessionError> {
        self.require_stopped("breakpoint.clear")?;
        let Some(index) = self
            .breakpoints
            .iter()
            .position(|breakpoint| breakpoint.id == id)
        else {
            return Err(DebugSessionError {
                kind: DebugErrorKind::UnknownBreakpoint,
                message: format!("debug breakpoint {id} does not exist"),
                hint: "Use an ID returned by breakpoint.set in this session.".to_string(),
            });
        };
        self.breakpoints.remove(index);
        Ok(())
    }

    fn require_stopped(&self, command: &'static str) -> Result<(), DebugSessionError> {
        if self.state == DebugSessionState::Stopped {
            Ok(())
        } else {
            Err(DebugSessionError::invalid_state(command, self.state))
        }
    }

    fn require_inspectable(&self, command: &'static str) -> Result<(), DebugSessionError> {
        if matches!(
            self.state,
            DebugSessionState::Stopped | DebugSessionState::Failed
        ) {
            Ok(())
        } else {
            Err(DebugSessionError::invalid_state(command, self.state))
        }
    }

    fn invalidate_inspection(&mut self) {
        self.inspection_generation = self.inspection_generation.wrapping_add(1).max(1);
        self.inspection =
            InspectionSnapshot::empty(self.inspection_generation, self.inspection_limits);
    }

    fn refresh_inspection(&mut self) {
        self.inspection = InspectionSnapshot::capture(
            &self.worker,
            self.inspection_generation,
            self.inspection_limits,
        );
    }
}

#[derive(Clone, Copy)]
enum ResumeMode {
    Continue,
    StepInto,
    StepOver,
    StepOut,
}

impl ResumeMode {
    const fn command(self) -> &'static str {
        match self {
            Self::Continue => "continue",
            Self::StepInto => "step_into",
            Self::StepOver => "step_over",
            Self::StepOut => "step_out",
        }
    }
}

fn stop_at_worker(
    executable: &VerifiedExecutable,
    worker: &Worker,
    reason: DebugStopReason,
    breakpoint_ids: Vec<u64>,
    diagnostic: Option<fpas_diagnostics::Diagnostic>,
) -> DebugStop {
    let instruction = if reason == DebugStopReason::RuntimeError {
        worker.current_address
    } else {
        InstructionAddress::try_from_index(worker.ip).unwrap_or(worker.current_address)
    };
    let point = breakpoints::point_at(executable, worker.function, instruction);
    DebugStop {
        reason,
        location: point.and_then(|point| breakpoints::source_location(executable, point)),
        instruction: instruction.get(),
        call_depth: worker.call_stack.len(),
        breakpoint_id: breakpoint_ids.first().copied(),
        breakpoint_ids,
        diagnostic,
    }
}

fn runtime_initialization_error(diagnostic: fpas_diagnostics::Diagnostic) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::InvalidState,
        message: format!("cannot initialize debug runtime: {}", diagnostic.message),
        hint: "Rebuild the executable with the current compiler and retry.".to_string(),
    }
}
