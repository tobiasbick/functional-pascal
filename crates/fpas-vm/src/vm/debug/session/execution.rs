//! Resume, task-aware stepping, pause, termination, and execution-limit control.

use std::time::Instant;

use super::*;
use crate::vm::debug::tasks::{DebugDispatch, DebugSchedule};

impl DebugSession {
    /// Continue all tasks until a breakpoint, pause, runtime failure, or termination.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state or execution-limit command error.
    pub fn continue_execution(&mut self) -> Result<DebugRunResult, DebugSessionError> {
        self.resume(ResumeMode::Continue, None)
    }

    /// Stop at the next sequence point in the task responsible for the current stop.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state, unknown-task, or execution-limit command error.
    pub fn step_into(&mut self) -> Result<DebugRunResult, DebugSessionError> {
        self.step_into_task(self.last_stop.task_id)
    }

    /// Stop at the next sequence point in `task_id`, including one inside a callee.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state, unknown-task, or execution-limit command error.
    pub fn step_into_task(&mut self, task_id: u64) -> Result<DebugRunResult, DebugSessionError> {
        self.resume(ResumeMode::StepInto, Some(task_id))
    }

    /// Stop at the next sequence point at the same or a lower call depth in the current task.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state, unknown-task, or execution-limit command error.
    pub fn step_over(&mut self) -> Result<DebugRunResult, DebugSessionError> {
        self.step_over_task(self.last_stop.task_id)
    }

    /// Stop over the current source expression in `task_id`.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state, unknown-task, or execution-limit command error.
    pub fn step_over_task(&mut self, task_id: u64) -> Result<DebugRunResult, DebugSessionError> {
        self.resume(ResumeMode::StepOver, Some(task_id))
    }

    /// Stop after returning below the current call depth in the current task.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state, unknown-task, or execution-limit command error.
    pub fn step_out(&mut self) -> Result<DebugRunResult, DebugSessionError> {
        self.step_out_task(self.last_stop.task_id)
    }

    /// Stop after `task_id` returns below its current call depth.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state, unknown-task, or execution-limit command error.
    pub fn step_out_task(&mut self, task_id: u64) -> Result<DebugRunResult, DebugSessionError> {
        self.resume(ResumeMode::StepOut, Some(task_id))
    }

    /// Terminate this owned session without executing more program instructions.
    pub fn disconnect(&mut self) {
        self.evaluation_cancelled.store(true, Ordering::Release);
        self.runtime.cancel();
        self.state = DebugSessionState::Terminated;
        self.pause_requested.store(false, Ordering::Release);
        self.debuggee.close();
        self.invalidate_inspection();
    }

    fn resume(
        &mut self,
        mode: ResumeMode,
        selected_task: Option<u64>,
    ) -> Result<DebugRunResult, DebugSessionError> {
        self.require_stopped(mode.command())?;
        let selected_task = selected_task.unwrap_or(self.last_stop.task_id);
        if mode != ResumeMode::Continue {
            if !self.runtime.task_is_inspectable(selected_task) {
                return Err(unknown_task(selected_task));
            }
            if self.runtime.task_is_paused(selected_task) {
                return Err(paused_step_error(mode.command(), selected_task));
            }
        }
        let starting_depth = if mode == ResumeMode::Continue {
            0
        } else {
            self.runtime
                .worker(selected_task)
                .map(|worker| worker.call_stack.len())
                .ok_or_else(|| unknown_task(selected_task))?
        };
        self.state = DebugSessionState::Running;
        self.invalidate_inspection();
        let mut pause_pending = false;
        let started = Instant::now();
        loop {
            if let Some(value) = self.runtime.take_finished_root_result() {
                let instruction_count = self.runtime.instruction_count();
                self.state = DebugSessionState::Terminated;
                self.inspections.clear();
                return Ok(DebugRunResult::Terminated(DebugTermination {
                    value,
                    instruction_count,
                }));
            }
            self.enforce_execution_limits(started)?;
            if self.pause_requested.swap(false, Ordering::AcqRel) {
                pause_pending = true;
            }
            let preferred = (mode != ResumeMode::Continue).then_some(selected_task);
            let schedule = match self.runtime.schedule(preferred) {
                Ok(schedule) => schedule,
                Err((task_id, diagnostic)) => {
                    return Ok(self.stop_for_runtime_error(task_id, diagnostic));
                }
            };
            let (task_id, resumed_at_boundary) = match schedule {
                DebugSchedule::Runnable {
                    task_id,
                    resumed_at_boundary,
                } => (task_id, resumed_at_boundary),
                DebugSchedule::Idle(wait) => {
                    if pause_pending {
                        return Ok(self.stop_for_reason(
                            selected_task,
                            DebugStopReason::Pause,
                            Vec::new(),
                        ));
                    }
                    self.runtime.wait(wait);
                    continue;
                }
                DebugSchedule::NoUnpausedWork => {
                    return Ok(self.stop_for_reason(
                        selected_task,
                        DebugStopReason::Pause,
                        Vec::new(),
                    ));
                }
            };
            if resumed_at_boundary
                && let Some(result) = self.stop_at_boundary(
                    task_id,
                    mode,
                    selected_task,
                    starting_depth,
                    pause_pending,
                )
            {
                return Ok(result);
            }
            let dispatch = match self.runtime.dispatch(task_id) {
                Ok(dispatch) => dispatch,
                Err((task_id, diagnostic)) => {
                    return Ok(self.stop_for_runtime_error(task_id, diagnostic));
                }
            };
            match dispatch {
                DebugDispatch::Completed {
                    task_id,
                    main: true,
                } => {
                    debug_assert_eq!(task_id, 0);
                    self.runtime.finish_main();
                    continue;
                }
                DebugDispatch::Completed {
                    task_id,
                    main: false,
                    ..
                } if mode != ResumeMode::Continue && task_id == selected_task => {
                    return Ok(self.stop_for_completed_task(task_id));
                }
                DebugDispatch::Completed { .. } => continue,
                DebugDispatch::Suspended(task_id)
                    if self.runtime.task_state(task_id)
                        != Some(super::super::types::DebugTaskState::Runnable) =>
                {
                    if pause_pending {
                        return Ok(self.stop_for_reason(
                            task_id,
                            DebugStopReason::Pause,
                            Vec::new(),
                        ));
                    }
                    continue;
                }
                DebugDispatch::Instruction(task_id) | DebugDispatch::Suspended(task_id) => {
                    if let Some(result) = self.stop_at_boundary(
                        task_id,
                        mode,
                        selected_task,
                        starting_depth,
                        pause_pending,
                    ) {
                        return Ok(result);
                    }
                }
            }
        }
    }

    fn stop_at_boundary(
        &mut self,
        task_id: u64,
        mode: ResumeMode,
        selected_task: u64,
        starting_depth: usize,
        pause_pending: bool,
    ) -> Option<DebugRunResult> {
        let sequence = self.next_sequence_point(task_id);
        if let Some((instruction, _)) = sequence {
            let mut breakpoint_ids = self
                .source_breakpoints
                .iter()
                .filter(|breakpoint| breakpoint.instruction == Some(instruction.get()))
                .map(|breakpoint| breakpoint.id)
                .collect::<Vec<_>>();
            breakpoint_ids.extend(
                self.function_breakpoints
                    .iter()
                    .filter(|breakpoint| breakpoint.instructions.contains(&instruction.get()))
                    .map(|breakpoint| breakpoint.id),
            );
            breakpoint_ids.sort_unstable();
            if !breakpoint_ids.is_empty() {
                return Some(self.stop_for_reason(
                    task_id,
                    DebugStopReason::Breakpoint,
                    breakpoint_ids,
                ));
            }
        }
        if pause_pending && sequence.is_some() {
            return Some(self.stop_for_reason(task_id, DebugStopReason::Pause, Vec::new()));
        }
        let _ = sequence?;
        if task_id != selected_task {
            return None;
        }
        let depth = self.runtime.worker(task_id)?.call_stack.len();
        let should_step = match mode {
            ResumeMode::Continue => false,
            ResumeMode::StepInto => true,
            ResumeMode::StepOver => depth <= starting_depth,
            ResumeMode::StepOut => depth < starting_depth,
        };
        should_step.then(|| self.stop_for_reason(task_id, DebugStopReason::Step, Vec::new()))
    }

    fn stop_for_reason(
        &mut self,
        task_id: u64,
        reason: DebugStopReason,
        breakpoint_ids: Vec<u64>,
    ) -> DebugRunResult {
        self.state = DebugSessionState::Stopped;
        let Some(worker) = self
            .runtime
            .worker(task_id)
            .or_else(|| self.runtime.worker(0))
        else {
            unreachable!("debug runtime always retains the main task")
        };
        self.last_stop = stop_at_worker(&self.executable, worker, reason, breakpoint_ids, None);
        self.last_stop.task_id = task_id;
        self.refresh_inspection();
        DebugRunResult::Stopped(self.last_stop.clone())
    }

    fn stop_for_runtime_error(
        &mut self,
        task_id: u64,
        diagnostic: fpas_diagnostics::Diagnostic,
    ) -> DebugRunResult {
        self.state = DebugSessionState::Failed;
        let Some(worker) = self
            .runtime
            .worker(task_id)
            .or_else(|| self.runtime.worker(0))
        else {
            unreachable!("debug runtime always retains the main task")
        };
        self.last_stop = stop_at_worker(
            &self.executable,
            worker,
            DebugStopReason::RuntimeError,
            Vec::new(),
            Some(diagnostic),
        );
        self.last_stop.task_id = task_id;
        self.refresh_inspection();
        DebugRunResult::Stopped(self.last_stop.clone())
    }

    fn stop_for_completed_task(&mut self, task_id: u64) -> DebugRunResult {
        self.state = DebugSessionState::Stopped;
        let instruction = self
            .runtime
            .worker(task_id)
            .map_or(0, |worker| worker.current_address.get());
        self.last_stop = DebugStop {
            reason: DebugStopReason::Step,
            task_id,
            location: None,
            instruction,
            call_depth: 0,
            breakpoint_id: None,
            breakpoint_ids: Vec::new(),
            diagnostic: None,
        };
        self.refresh_inspection();
        DebugRunResult::Stopped(self.last_stop.clone())
    }

    fn enforce_execution_limits(&mut self, started: Instant) -> Result<(), DebugSessionError> {
        let instruction_count = self.runtime.instruction_count();
        let (kind, message, hint) = if instruction_count >= self.execution_limits.max_instructions {
            (
                DebugErrorKind::InstructionLimit,
                format!(
                    "debug execution exceeded the {} instruction limit",
                    self.execution_limits.max_instructions
                ),
                "Increase the debug instruction limit or inspect the program for a non-terminating loop.",
            )
        } else if started.elapsed() >= self.execution_limits.timeout {
            (
                DebugErrorKind::ExecutionTimeout,
                format!(
                    "debug execution exceeded the {} ms timeout",
                    self.execution_limits.timeout.as_millis()
                ),
                "Increase the debug timeout or pause execution sooner.",
            )
        } else if self.output_byte_count() > self.execution_limits.max_output_bytes {
            (
                DebugErrorKind::OutputLimit,
                format!(
                    "debug execution exceeded the {} byte output limit",
                    self.execution_limits.max_output_bytes
                ),
                "Increase the debug output limit or reduce program output.",
            )
        } else {
            return Ok(());
        };
        self.state = DebugSessionState::Failed;
        self.refresh_inspection();
        Err(DebugSessionError {
            kind,
            message,
            hint: hint.to_string(),
        })
    }

    fn output_byte_count(&self) -> usize {
        let Some(worker) = self.runtime.worker(0) else {
            return 0;
        };
        let output = worker
            .hosted
            .console
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        output.output().lines.iter().fold(0usize, |total, line| {
            total.saturating_add(line.len()).saturating_add(1)
        })
    }

    fn next_sequence_point(
        &self,
        task_id: u64,
    ) -> Option<(InstructionAddress, &fpas_bytecode::SequencePoint)> {
        let worker = self.runtime.worker(task_id)?;
        let instruction = InstructionAddress::try_from_index(worker.ip).ok()?;
        let point = crate::vm::debug::breakpoints::point_at(
            &self.executable,
            worker.function,
            instruction,
        )?;
        Some((instruction, point))
    }
}
