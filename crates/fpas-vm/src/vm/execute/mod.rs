mod aggregates;
mod concurrency;
mod control_calls;
mod enums;
mod io;
mod numeric;
mod result_option;
mod stack_scope;

use super::diagnostics::VmError;
use super::{Worker, internal_error, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::{RUNTIME_PROGRAM_PANIC, RUNTIME_VM_SHUTDOWN};

/// Outcome of executing a single instruction via [`Worker::exec_one`].
pub(super) enum StepResult {
    /// Normal instruction executed; continue.
    Continue,
    /// `Op::Return` was decoded — caller must handle stack frame.
    Return,
    /// `Op::Halt` was decoded.
    Halt,
}

impl Worker {
    /// Fetch, decode and execute the next instruction.
    ///
    /// All opcodes **except** `Return`, `Halt`, and `Panic` are fully handled
    /// here. `Return`/`Halt` are signalled back via [`StepResult`] so that the
    /// two loop drivers (`run` and `call_function_sync`) can apply their own
    /// control-flow logic. `Panic` always returns an `Err`.
    pub(super) fn exec_one(
        &mut self,
        fallback_line: SourceLocation,
    ) -> Result<StepResult, VmError> {
        let op = self.shared.chunk.code[self.ip];
        let line = self
            .shared
            .chunk
            .location_at(self.ip)
            .unwrap_or(fallback_line);
        self.current_location = line;
        self.ip += 1;

        if self.try_exec_stack_scope(op, line)?
            || self.try_exec_numeric(op, line)?
            || self.try_exec_control_calls(op, line)?
            || self.try_exec_concurrency(op, line)?
            || self.try_exec_aggregates(op, line)?
            || self.try_exec_result_option(op, line)?
            || self.try_exec_enums(op, line)?
            || self.try_exec_io(op, line)?
        {
            self.maybe_timeslice_yield();
            return Ok(StepResult::Continue);
        }

        match op {
            Op::Return => Ok(StepResult::Return),
            Op::Halt => Ok(StepResult::Halt),
            Op::Panic => {
                let val = self.pop(line)?;
                Err(runtime_error(
                    RUNTIME_PROGRAM_PANIC,
                    format!("panic: {val}"),
                    "Remove the panic or guard the failing condition before calling panic.",
                    line,
                ))
            }
            _ => Err(internal_error(
                format!("Unhandled opcode in VM dispatcher: {op:?}"),
                "This indicates a VM dispatch bug. Please report it.",
                line,
            )),
        }
    }

    pub fn run(&mut self) -> Result<(), VmError> {
        loop {
            if self.shared.is_shutdown() && self.current_task_id == 0 {
                return Err(runtime_error(
                    RUNTIME_VM_SHUTDOWN,
                    "Execution aborted: a concurrent task failed",
                    "A task spawned with `go` raised a runtime error. Fix the error in the spawned task.",
                    self.current_location,
                ));
            }

            let code_len = self.shared.chunk.code.len();
            if self.ip == code_len {
                if self.current_task_id != 0 {
                    let result = self.stack.pop().unwrap_or(Value::Unit);
                    if self.current_task_retain_result {
                        self.shared.store_task_result(self.current_task_id, result);
                    }
                    if self.pick_next_task() {
                        continue;
                    }
                }
                return Ok(());
            }
            if self.ip > code_len {
                return Err(internal_error(
                    format!(
                        "Instruction pointer jumped past the end of the chunk: ip={}, len={code_len}",
                        self.ip
                    ),
                    "This indicates malformed bytecode or a VM control-flow bug. Please report it.",
                    self.current_location,
                ));
            }

            match self.exec_one(self.current_location)? {
                StepResult::Continue => {}
                StepResult::Halt => return Ok(()),
                StepResult::Return => {
                    let line = self.current_location;
                    let return_val = self.pop(line)?;
                    if let Some(frame) = self.call_stack.pop() {
                        self.stack.truncate(frame.base_slot);
                        self.push(return_val)?;
                        self.ip = frame.return_ip;
                    } else if self.current_task_id == 0 {
                        return Ok(());
                    } else {
                        if self.current_task_retain_result {
                            self.shared
                                .store_task_result(self.current_task_id, return_val);
                        }
                        if !self.pick_next_task() {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    fn pick_next_task(&mut self) -> bool {
        if let Some(task) = self.shared.try_dequeue_task() {
            self.load_task(task);
            true
        } else {
            false
        }
    }
}
