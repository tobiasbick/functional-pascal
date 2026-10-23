//! Root and scheduled execution loops over bounded dispatch batches.

use fpas_bytecode::Value;
use fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN;

use super::Worker;
use crate::vm::dispatch::DispatchStep;
use crate::vm::{RuntimeExecution, VmError, diagnostics};

/// Maximum scheduling steps between two scheduler abort checks.
const ABORT_CHECK_INTERVAL: u32 = crate::vm::BATCH_STEPS;

impl Worker {
    /// Execute one scheduled task until return, suspension or scheduler abort.
    pub(in crate::vm) fn run_task(&mut self) -> Result<Option<Value>, VmError> {
        let access = self.execution_access()?;
        let result = self.run_task_owned();
        if access.outermost() {
            let extra = match &result {
                Ok(Some(value)) => std::slice::from_ref(value),
                _ => &[],
            };
            if self.publish_execution(extra).is_ok() {
                access.safe_point();
            }
        }
        result
    }
    fn run_task_owned(&mut self) -> Result<Option<Value>, VmError> {
        if self.capture_record.is_some() {
            self.run_task_inner::<true>()
        } else {
            self.run_task_inner::<false>()
        }
    }

    fn run_task_inner<const CAPTURE: bool>(&mut self) -> Result<Option<Value>, VmError> {
        if let Some(outcome) = self.pending_completion.take() {
            return outcome.map(Some);
        }
        loop {
            if CAPTURE {
                self.capture_peer_boundary();
            }
            if self
                .scheduler
                .as_ref()
                .is_some_and(|scheduler| scheduler.is_aborted())
            {
                return Ok(None);
            }
            let scheduled = self.task_id != 0;
            let budget = if scheduled {
                self.instructions_until_yield.clamp(1, ABORT_CHECK_INTERVAL)
            } else {
                ABORT_CHECK_INTERVAL
            };
            let batch = self
                .dispatch_batch::<false>(budget)
                .map_err(|error| self.capture_failure(error))?;
            match batch.step {
                DispatchStep::Continue => {}
                DispatchStep::Suspend => return Ok(None),
                DispatchStep::Return(value) => return Ok(Some(value)),
            }
            if scheduled {
                self.instructions_until_yield =
                    self.instructions_until_yield.saturating_sub(batch.steps);
                if self.instructions_until_yield == 0 {
                    // Yielding saves and republishes the whole task. It only helps when other
                    // work is waiting; the check repeats at every slice, so waiters never starve.
                    if !CAPTURE
                        && self
                            .scheduler
                            .as_ref()
                            .is_some_and(|scheduler| !scheduler.has_queued())
                    {
                        self.instructions_until_yield = crate::vm::TIMESLICE;
                    } else {
                        self.suspend_and_enqueue();
                        return Ok(None);
                    }
                }
            }
        }
    }

    /// Execute and consume a standalone worker through its ownership barriers.
    pub fn run(mut self) -> Result<RuntimeExecution, VmError> {
        self.run_retained()
    }

    /// Execute while retaining ownership of root storage through VM finalization.
    pub(in crate::vm) fn run_retained(&mut self) -> Result<RuntimeExecution, VmError> {
        let access = self.execution_access()?;
        let execution = self.run_in_place();
        let outcome = execution
            .as_ref()
            .map(|execution| execution.value.clone())
            .map_err(Clone::clone);
        let value = match self.finish_owned_groups_blocking(outcome) {
            Ok(value) => value,
            Err(error) => {
                if self.publish_execution(&[]).is_ok() {
                    access.safe_point();
                }
                return Err(error);
            }
        };
        let execution = RuntimeExecution::rooted(
            value,
            self.instruction_count
                .saturating_add(self.callback_instruction_count.get()),
            &self.domain,
        );
        if self
            .publish_execution(std::slice::from_ref(&execution.value))
            .is_ok()
        {
            access.safe_point();
        }
        Ok(execution)
    }

    /// Execute a synchronous root/callback, retaining failure state before scope unwind.
    pub(in crate::vm) fn run_in_place(&mut self) -> Result<RuntimeExecution, VmError> {
        let access = self.execution_access()?;
        let result = self.run_in_place_inner().map_err(|error| {
            let error = self.capture_failure(error);
            self.capture_root_propagation(&error);
            self.unwind_task_scopes_blocking(error)
        });
        if access.outermost() {
            let extra = match &result {
                Ok(execution) => std::slice::from_ref(&execution.value),
                Err(_) => &[],
            };
            if self.publish_execution(extra).is_ok() {
                access.safe_point();
            }
        }
        result
    }

    fn run_in_place_inner(&mut self) -> Result<RuntimeExecution, VmError> {
        if self.capture_record.is_some() {
            self.run_in_place_loop::<true>()
        } else {
            self.run_in_place_loop::<false>()
        }
    }

    fn run_in_place_loop<const CAPTURE: bool>(&mut self) -> Result<RuntimeExecution, VmError> {
        loop {
            if CAPTURE {
                self.capture_peer_boundary();
            }
            if self
                .scheduler
                .as_ref()
                .is_some_and(|scheduler| scheduler.is_aborted())
            {
                return Err(diagnostics::at_address(
                    self.executable.executable(),
                    self.current_address,
                    RUNTIME_VM_SHUTDOWN,
                    "Register VM execution was canceled",
                    "Create a new VM instance to run the program again.",
                ));
            }
            match self.dispatch_batch::<false>(ABORT_CHECK_INTERVAL)?.step {
                DispatchStep::Continue => {}
                DispatchStep::Suspend => {
                    return Err(diagnostics::internal(
                        self.executable.executable(),
                        self.current_address,
                        "Root register execution suspended unexpectedly",
                    ));
                }
                DispatchStep::Return(value) => {
                    return Ok(RuntimeExecution::rooted(
                        value,
                        self.instruction_count
                            .saturating_add(self.callback_instruction_count.get()),
                        &self.domain,
                    ));
                }
            }
        }
    }
}
