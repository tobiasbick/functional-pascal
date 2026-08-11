//! Resume, stepping, pause, termination, and execution-limit control.

use std::time::Instant;

use crate::vm::dispatch::DispatchStep;

use super::*;

impl DebugSession {
    /// Continue until a breakpoint, pause, runtime failure, or termination.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state command error.
    pub fn continue_execution(&mut self) -> Result<DebugRunResult, DebugSessionError> {
        self.resume(ResumeMode::Continue)
    }

    /// Stop at the next sequence point, including one inside a callee.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state command error.
    pub fn step_into(&mut self) -> Result<DebugRunResult, DebugSessionError> {
        self.resume(ResumeMode::StepInto)
    }

    /// Stop at the next sequence point at the same or a lower call depth.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state command error.
    pub fn step_over(&mut self) -> Result<DebugRunResult, DebugSessionError> {
        self.resume(ResumeMode::StepOver)
    }

    /// Stop at the first sequence point below the current call depth.
    ///
    /// # Errors
    ///
    /// Returns an invalid-state command error.
    pub fn step_out(&mut self) -> Result<DebugRunResult, DebugSessionError> {
        self.resume(ResumeMode::StepOut)
    }

    /// Terminate this owned session without executing more program instructions.
    pub fn disconnect(&mut self) {
        self.evaluation_cancelled.store(true, Ordering::Release);
        self.state = DebugSessionState::Terminated;
        self.pause_requested.store(false, Ordering::Release);
        self.invalidate_inspection();
    }

    fn resume(&mut self, mode: ResumeMode) -> Result<DebugRunResult, DebugSessionError> {
        self.require_stopped(mode.command())?;
        let starting_depth = self.worker.call_stack.len();
        self.state = DebugSessionState::Running;
        self.invalidate_inspection();
        let mut pause_pending = false;
        let started = Instant::now();
        loop {
            self.enforce_execution_limits(started)?;
            if self.pause_requested.swap(false, Ordering::AcqRel) {
                pause_pending = true;
            }
            let dispatch = match self.worker.dispatch_one() {
                Ok(dispatch) => dispatch,
                Err(diagnostic) => {
                    self.state = DebugSessionState::Failed;
                    self.last_stop = stop_at_worker(
                        &self.executable,
                        &self.worker,
                        DebugStopReason::RuntimeError,
                        Vec::new(),
                        Some(diagnostic),
                    );
                    self.refresh_inspection();
                    return Ok(DebugRunResult::Stopped(self.last_stop.clone()));
                }
            };
            match dispatch {
                DispatchStep::Continue => {}
                DispatchStep::Suspend => {
                    unreachable!("task-spawning executables are rejected before debug execution")
                }
                DispatchStep::Return(value) => {
                    self.state = DebugSessionState::Terminated;
                    self.inspection = InspectionSnapshot::empty(
                        Arc::clone(&self.executable),
                        self.inspection_generation,
                        self.inspection_limits,
                    );
                    return Ok(DebugRunResult::Terminated(DebugTermination {
                        value,
                        instruction_count: self.worker.instruction_count,
                    }));
                }
            }
            let Some((instruction, point)) = self.next_sequence_point() else {
                continue;
            };
            let breakpoint_ids = self
                .breakpoints
                .iter()
                .filter(|breakpoint| breakpoint.instruction == Some(instruction.get()))
                .map(|breakpoint| breakpoint.id)
                .collect::<Vec<_>>();
            if !breakpoint_ids.is_empty() {
                self.state = DebugSessionState::Stopped;
                self.last_stop = stop_at_worker(
                    &self.executable,
                    &self.worker,
                    DebugStopReason::Breakpoint,
                    breakpoint_ids,
                    None,
                );
                self.refresh_inspection();
                return Ok(DebugRunResult::Stopped(self.last_stop.clone()));
            }
            if pause_pending {
                self.state = DebugSessionState::Stopped;
                self.last_stop = stop_at_worker(
                    &self.executable,
                    &self.worker,
                    DebugStopReason::Pause,
                    Vec::new(),
                    None,
                );
                self.refresh_inspection();
                return Ok(DebugRunResult::Stopped(self.last_stop.clone()));
            }
            let depth = self.worker.call_stack.len();
            let should_step = match mode {
                ResumeMode::Continue => false,
                ResumeMode::StepInto => true,
                ResumeMode::StepOver => depth <= starting_depth,
                ResumeMode::StepOut => depth < starting_depth,
            };
            if should_step {
                debug_assert_eq!(point.instruction, instruction);
                self.state = DebugSessionState::Stopped;
                self.last_stop = stop_at_worker(
                    &self.executable,
                    &self.worker,
                    DebugStopReason::Step,
                    Vec::new(),
                    None,
                );
                self.refresh_inspection();
                return Ok(DebugRunResult::Stopped(self.last_stop.clone()));
            }
        }
    }

    fn enforce_execution_limits(&mut self, started: Instant) -> Result<(), DebugSessionError> {
        let (kind, message, hint) = if self.worker.instruction_count
            >= self.execution_limits.max_instructions
        {
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
        let output = self
            .worker
            .hosted
            .console
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        output.output().lines.iter().fold(0usize, |total, line| {
            total.saturating_add(line.len()).saturating_add(1)
        })
    }

    fn next_sequence_point(&self) -> Option<(InstructionAddress, &fpas_bytecode::SequencePoint)> {
        let instruction = InstructionAddress::try_from_index(self.worker.ip).ok()?;
        let point = breakpoints::point_at(&self.executable, self.worker.function, instruction)?;
        Some((instruction, point))
    }
}
