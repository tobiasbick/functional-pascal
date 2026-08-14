//! Debug session construction, state transitions, and controlled dispatch.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

use fpas_bytecode::{InstructionAddress, VerifiedExecutable};

use super::breakpoints::{self, BoundBreakpoint, SourceBreakpoint};
use super::inspection::{DebugInspectionLimits, InspectionSnapshot};
use super::tasks::DebugTaskRuntime;
use super::types::{
    DebugErrorKind, DebugExecutionLimits, DebugRunResult, DebugSessionError, DebugSessionState,
    DebugStop, DebugStopReason, DebugTermination,
};
use crate::vm::hosted::HostedState;
use crate::vm::layouts::RuntimeLayouts;
use crate::vm::tasks::{DebugClock, TaskScheduler};
use crate::vm::worker::Worker;

mod dictionary;
mod execution;
mod forced_return;
mod inspection;
mod mutation;
mod sequence;
mod storage;
mod variant;

/// Thread-safe cooperative pause request handle.
#[derive(Clone)]
pub struct DebugPauseHandle {
    requested: Arc<AtomicBool>,
}

/// Thread-safe cooperative cancellation handle for active debugger call evaluation.
#[derive(Clone)]
pub struct DebugEvaluationCancelHandle {
    cancelled: Arc<AtomicBool>,
}

impl DebugEvaluationCancelHandle {
    /// Cancel the active or next debugger call evaluation at an instruction boundary.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
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
    runtime: DebugTaskRuntime,
    state: DebugSessionState,
    breakpoints: Vec<BoundBreakpoint>,
    next_breakpoint_id: u64,
    pause_requested: Arc<AtomicBool>,
    evaluation_cancelled: Arc<AtomicBool>,
    last_stop: DebugStop,
    inspection_generation: u32,
    inspection_task_id: u64,
    inspections: BTreeMap<u64, InspectionSnapshot>,
    inspection_limits: DebugInspectionLimits,
    execution_limits: DebugExecutionLimits,
}

impl DebugSession {
    /// Construct a stopped debug session at the program entry.
    ///
    /// # Errors
    ///
    /// Returns an actionable error when runtime state cannot be initialized.
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
    /// Returns an actionable error when runtime state cannot be initialized.
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
    /// Returns an actionable error for invalid runtime state.
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
    /// Returns an actionable error for invalid runtime state.
    pub fn with_limits(
        executable: VerifiedExecutable,
        arguments: Vec<String>,
        inspection_limits: DebugInspectionLimits,
        execution_limits: DebugExecutionLimits,
    ) -> Result<Self, DebugSessionError> {
        Self::with_debug_clock(
            executable,
            arguments,
            inspection_limits,
            execution_limits,
            Arc::new(DebugClock::realtime()),
        )
    }

    #[cfg(test)]
    /// Construct a session whose timer waits advance a deterministic manual clock.
    pub(in crate::vm::debug) fn with_manual_clock(
        executable: VerifiedExecutable,
    ) -> Result<Self, DebugSessionError> {
        Self::with_debug_clock(
            executable,
            Vec::new(),
            DebugInspectionLimits::default(),
            DebugExecutionLimits::default(),
            Arc::new(DebugClock::manual()),
        )
    }

    fn with_debug_clock(
        executable: VerifiedExecutable,
        arguments: Vec<String>,
        inspection_limits: DebugInspectionLimits,
        execution_limits: DebugExecutionLimits,
        debug_clock: Arc<DebugClock>,
    ) -> Result<Self, DebugSessionError> {
        let executable = Arc::new(executable);
        let globals = Arc::new(RwLock::new(vec![
            None;
            executable.executable().globals.len()
        ]));
        let layouts = RuntimeLayouts::build(executable.executable(), InstructionAddress::new(0))
            .map(Arc::new)
            .map_err(runtime_initialization_error)?;
        let hosted = Arc::new(HostedState::new(fpas_std::Console::new(), arguments));
        let scheduler = Arc::new(TaskScheduler::new());
        let worker = Worker::for_function_with_state(
            Arc::clone(&executable),
            executable.executable().entry,
            Vec::new(),
            globals,
            layouts,
            hosted,
        )
        .map_err(runtime_initialization_error)?
        .with_scheduler(Some(Arc::clone(&scheduler)))
        .with_debug_tasks(Arc::clone(&debug_clock));
        let pause_requested = Arc::new(AtomicBool::new(false));
        let evaluation_cancelled = Arc::new(AtomicBool::new(false));
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
        let inspections = BTreeMap::from([(0, inspection)]);
        let runtime = DebugTaskRuntime::new(worker, scheduler, debug_clock);
        Ok(Self {
            executable,
            runtime,
            state: DebugSessionState::Stopped,
            breakpoints: Vec::new(),
            next_breakpoint_id: 1,
            pause_requested,
            evaluation_cancelled,
            last_stop,
            inspection_generation,
            inspection_task_id: 0,
            inspections,
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
        let Some(worker) = self.runtime.worker(0) else {
            unreachable!("debug runtime always retains the main task")
        };
        worker
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

    /// Create a handle that can cancel controlled call evaluation.
    #[must_use]
    pub fn evaluation_cancel_handle(&self) -> DebugEvaluationCancelHandle {
        DebugEvaluationCancelHandle {
            cancelled: Arc::clone(&self.evaluation_cancelled),
        }
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
        self.inspections.clear();
    }

    fn refresh_inspection(&mut self) {
        self.refresh_inspection_with_reserved_handles(None);
    }

    fn refresh_inspection_with_reserved_handles(&mut self, reservation: Option<(u64, usize)>) {
        let task_id = self.last_stop.task_id;
        self.inspections.clear();
        for inspectable_task_id in self.runtime.inspectable_task_ids() {
            let Some(worker) = self.runtime.worker(inspectable_task_id) else {
                continue;
            };
            self.inspection_generation = self.inspection_generation.wrapping_add(1).max(1);
            let reserved_handles = reservation
                .filter(|(reserved_task_id, _)| *reserved_task_id == inspectable_task_id)
                .map_or(0, |(_, count)| count);
            self.inspections.insert(
                inspectable_task_id,
                InspectionSnapshot::capture_with_reserved_handles(
                    worker,
                    self.inspection_generation,
                    self.inspection_limits,
                    reserved_handles,
                ),
            );
        }
        self.inspection_task_id = if self.inspections.contains_key(&task_id) {
            task_id
        } else {
            self.inspections.keys().next().copied().unwrap_or(task_id)
        };
    }

    fn select_inspection_task(&mut self, task_id: u64) -> Result<(), DebugSessionError> {
        if !self.inspections.contains_key(&task_id) {
            return Err(unknown_task(task_id));
        }
        self.inspection_task_id = task_id;
        Ok(())
    }

    fn current_inspection(&self) -> Result<&InspectionSnapshot, DebugSessionError> {
        self.inspections
            .get(&self.inspection_task_id)
            .ok_or_else(|| unknown_task(self.inspection_task_id))
    }

    fn inspection_for_item(&self, id: u64) -> Result<&InspectionSnapshot, DebugSessionError> {
        let generation = (id >> 32) as u32;
        self.inspections
            .values()
            .find(|inspection| inspection.generation() == generation)
            .ok_or_else(|| DebugSessionError {
                kind: DebugErrorKind::UnknownFrame,
                message: format!("debug frame {id} is unknown or expired"),
                hint: "Request stack frames again for the current stop.".to_string(),
            })
    }

    fn inspection_for_item_mut(
        &mut self,
        id: u64,
    ) -> Result<&mut InspectionSnapshot, DebugSessionError> {
        let generation = (id >> 32) as u32;
        self.inspections
            .values_mut()
            .find(|inspection| inspection.generation() == generation)
            .ok_or_else(|| DebugSessionError {
                kind: DebugErrorKind::UnknownVariablesReference,
                message: format!("debug variables reference {id} is unknown or expired"),
                hint: "Request scopes or parent variables again for the current stop.".to_string(),
            })
    }

    fn task_for_frame(&self, frame_id: Option<u64>) -> Result<u64, DebugSessionError> {
        let Some(frame_id) = frame_id else {
            return Ok(self.inspection_task_id);
        };
        let generation = (frame_id >> 32) as u32;
        self.inspections
            .iter()
            .find_map(|(&task_id, inspection)| {
                (inspection.generation() == generation).then_some(task_id)
            })
            .ok_or_else(|| DebugSessionError {
                kind: DebugErrorKind::UnknownFrame,
                message: format!("debug frame {frame_id} is unknown or expired"),
                hint: "Request stack frames again for the current stop.".to_string(),
            })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
        task_id: worker.task_id,
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

fn unknown_task(task_id: u64) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::UnknownTask,
        message: format!("debug task {task_id} is unknown or no longer inspectable"),
        hint: "Request the current task list and select an inspectable task.".to_string(),
    }
}
