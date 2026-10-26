//! Batched dispatch of pre-decoded verified instructions.
//!
//! A batch keeps executing instructions without returning to the scheduling loop. Only opcodes
//! that can suspend the task or advance a hosted callback operation (`Return`, `Intrinsic`,
//! `SpawnTask`, `Yield`) are followed by suspension and callback checks.

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
    /// Keep dispatching without changing hosted callback readiness.
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
        let access = self.execution_access()?;
        let started = self.hosted.profile.get().map(|_| std::time::Instant::now());
        let result = self.dispatch_owned::<SUPPRESS_INITIALIZERS>(budget);
        if let (Some(started), Ok(batch)) = (started, &result) {
            self.record_profile_batch(batch.steps, started.elapsed());
        }
        access.note_completed_batch();
        if access.outermost() || self.domain.maintenance_pending() {
            let _timer = fpas_bytecode::profile::Timer::start(
                fpas_bytecode::profile::Counter::SafePointNanos,
            );
            let extra = match &result {
                Ok(batch) => match &batch.step {
                    DispatchStep::Return(value) => std::slice::from_ref(value),
                    _ => &[],
                },
                Err(_) => &[],
            };
            // A failed publication defers maintenance; it does not replace program diagnostics.
            if self.publish_execution(extra).is_ok() {
                access.safe_point();
            }
        }
        result
    }

    fn dispatch_owned<const SUPPRESS_INITIALIZERS: bool>(
        &mut self,
        budget: u32,
    ) -> Result<DispatchBatch, VmError> {
        // Trace hooks must not enter the ordinary per-instruction hot path. Select once
        // per batch; synchronous callbacks can inherit traces without debug_tasks.
        if self.debug_trace.is_some() || self.debug_local_trace.is_some() {
            self.dispatch_batch_inner::<SUPPRESS_INITIALIZERS, true>(budget)
        } else {
            self.dispatch_batch_inner::<SUPPRESS_INITIALIZERS, false>(budget)
        }
    }

    fn dispatch_batch_inner<const SUPPRESS_INITIALIZERS: bool, const TRACE: bool>(
        &mut self,
        budget: u32,
    ) -> Result<DispatchBatch, VmError> {
        self.check_task_cancellation()?;
        let mut steps = 0;
        'batch: while steps < budget {
            self.check_task_cancellation()?;
            // A ready hosted continuation advances as one scheduling step inside the batch,
            // so each callback does not pay a separate execution-access round trip.
            if self.callback_resume_pending() {
                if TRACE {
                    self.trace_local_begin();
                }
                let (owner, address) = self
                    .callback_origin()
                    .unwrap_or((self.function, self.current_address));
                let result = self.resume_callback_continuation();
                // Continuation work has no declaration store, even if current_address still
                // names the last completed Move in a callback frame.
                if TRACE {
                    self.trace_local_finish(owner, address, false, false, false);
                }
                if result? {
                    steps += 1;
                    if self.suspend_requested && self.task_suspension.is_some() {
                        return Ok(DispatchBatch {
                            step: DispatchStep::Suspend,
                            steps,
                        });
                    }
                    // Requested maintenance is serviced at this boundary, as before.
                    if self.domain.maintenance_pending() {
                        break;
                    }
                    continue;
                }
            }
            // Run ordinary instructions without inspecting callback state again. Only
            // boundary opcodes can make a continuation ready, so return to the outer loop there.
            loop {
                if TRACE && self.trace_interrupted() {
                    return Err(diagnostics::at_address(
                        self.executable.executable(),
                        self.current_address,
                        fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN,
                        "Debug trace collection reached a bound",
                        "Inspect the bounded batch trace result.",
                    ));
                }
                let Some(&instruction) = self.executable.decoded().get(self.ip) else {
                    return Err(self.invalid_instruction_pointer());
                };
                // Verified code length fits an instruction address.
                self.current_address = InstructionAddress::new(self.ip as u32);
                if TRACE {
                    self.trace_local_begin();
                }
                if TRACE && self.trace_interrupted() {
                    return Err(diagnostics::at_address(
                        self.executable.executable(),
                        self.current_address,
                        fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN,
                        "Debug trace reached a bound",
                        "Inspect trace coverage.",
                    ));
                }
                self.ip += 1;
                self.instruction_count = self.instruction_count.wrapping_add(1);
                steps += 1;
                if SUPPRESS_INITIALIZERS
                    && self.take_suppressed_source_initializer(self.current_address)
                {
                    break 'batch;
                }
                let owner = self.function;
                let address = self.current_address;
                let old_depth = self.call_stack.len();
                let result = self.execute(instruction);
                if TRACE {
                    let tail = matches!(
                        instruction.opcode(),
                        fpas_bytecode::Opcode::TailCall | fpas_bytecode::Opcode::TailCallValue
                    ) && self.call_stack.len() == old_depth
                        && result.is_ok();
                    self.trace_local_finish(
                        owner,
                        address,
                        result.is_ok(),
                        matches!(result, Ok(Flow::Return(_))),
                        tail,
                    );
                }
                match result? {
                    Flow::Next => {}
                    Flow::Boundary => {
                        if self.suspend_requested {
                            return Ok(DispatchBatch {
                                step: DispatchStep::Suspend,
                                steps,
                            });
                        }
                        if self.domain.maintenance_pending() && self.callback_resume_pending() {
                            break 'batch;
                        }
                        break;
                    }
                    Flow::Return(value) => {
                        return Ok(DispatchBatch {
                            step: DispatchStep::Return(value),
                            steps,
                        });
                    }
                }
                if steps >= budget {
                    break 'batch;
                }
                self.check_task_cancellation()?;
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
