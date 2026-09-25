//! Batched dispatch of pre-decoded verified instructions.
//!
//! A batch keeps executing instructions without returning to the scheduling loop. Only opcodes
//! that can suspend the task or advance a hosted callback operation (`Return`, `Intrinsic`,
//! `SpawnTask`, `SpawnDetachedTask`, `Yield`) are followed by suspension and callback checks.

mod opcodes;

use fpas_bytecode::{InstructionAddress, Value};

use super::worker::Worker;
use super::{VmError, diagnostics};

pub(super) enum DispatchStep {
    Continue,
    Suspend,
    Return(Value),
}

/// Outcome of one bounded dispatch batch.
pub(super) struct DispatchBatch {
    pub(super) step: DispatchStep,
    /// Scheduling steps consumed: executed instructions plus callback-continuation steps.
    pub(super) steps: u32,
}

/// Control effect of one executed instruction inside a batch.
enum Flow {
    /// Keep dispatching in the current batch.
    Next,
    /// The opcode may have requested suspension or readied a hosted callback continuation.
    Boundary,
    /// The root frame returned this value.
    Return(Value),
}

impl Worker {
    /// Execute one instruction without debugger-only initializer suppression.
    pub fn dispatch_one(&mut self) -> Result<DispatchStep, VmError> {
        self.dispatch_batch::<false>(1).map(|batch| batch.step)
    }

    /// Execute one instruction for debugger-owned execution.
    pub(in crate::vm) fn dispatch_debug_one(&mut self) -> Result<DispatchStep, VmError> {
        self.dispatch_batch::<true>(1).map(|batch| batch.step)
    }

    /// Execute up to `budget` scheduling steps and stop early at suspension, return, or a ready
    /// hosted callback continuation.
    pub(in crate::vm) fn dispatch_batch<const SUPPRESS_INITIALIZERS: bool>(
        &mut self,
        budget: u32,
    ) -> Result<DispatchBatch, VmError> {
        if !self.callback_continuations.is_empty() && self.resume_callback_continuation()? {
            return Ok(DispatchBatch {
                step: DispatchStep::Continue,
                steps: 1,
            });
        }
        let mut steps = 0;
        while steps < budget {
            let Some(&instruction) = self.executable.decoded().get(self.ip) else {
                return Err(self.invalid_instruction_pointer());
            };
            // Verified code length fits an instruction address.
            self.current_address = InstructionAddress::new(self.ip as u32);
            self.ip += 1;
            self.instruction_count = self.instruction_count.wrapping_add(1);
            steps += 1;
            if SUPPRESS_INITIALIZERS
                && self.take_suppressed_source_initializer(self.current_address)
            {
                break;
            }
            match self.execute(instruction)? {
                Flow::Next => {}
                Flow::Boundary => {
                    if self.suspend_requested {
                        return Ok(DispatchBatch {
                            step: DispatchStep::Suspend,
                            steps,
                        });
                    }
                    if self.callback_resume_pending() {
                        break;
                    }
                }
                Flow::Return(value) => {
                    return Ok(DispatchBatch {
                        step: DispatchStep::Return(value),
                        steps,
                    });
                }
            }
        }
        Ok(DispatchBatch {
            step: DispatchStep::Continue,
            steps,
        })
    }

    #[cold]
    #[inline(never)]
    fn invalid_instruction_pointer(&self) -> VmError {
        let address = InstructionAddress::try_from_index(self.ip).unwrap_or(self.current_address);
        diagnostics::internal(
            self.executable.executable(),
            address,
            "Instruction pointer is outside verified code",
        )
    }
}
