//! Root and scheduled execution loops over bounded dispatch batches.

use fpas_bytecode::Value;
use fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN;

use super::Worker;
use crate::vm::dispatch::DispatchStep;
use crate::vm::{Execution, VmError, diagnostics};

/// Maximum scheduling steps between two scheduler abort checks.
const ABORT_CHECK_INTERVAL: u32 = 256;

impl Worker {
    pub(in crate::vm) fn run_task(&mut self) -> Result<Option<Value>, VmError> {
        loop {
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
            let batch = self.dispatch_batch::<false>(budget)?;
            match batch.step {
                DispatchStep::Continue => {}
                DispatchStep::Suspend => return Ok(None),
                DispatchStep::Return(value) => return Ok(Some(value)),
            }
            if scheduled {
                self.instructions_until_yield =
                    self.instructions_until_yield.saturating_sub(batch.steps);
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

    pub(in crate::vm) fn run_in_place(&mut self) -> Result<Execution, VmError> {
        loop {
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
}
