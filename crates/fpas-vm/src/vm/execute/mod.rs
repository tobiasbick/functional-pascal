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

impl Worker {
    pub fn run(&mut self) -> Result<(), VmError> {
        loop {
            // Abort if another worker signalled shutdown (e.g. panic in a task).
            if self.shared.is_shutdown() && self.current_task_id == 0 {
                return Err(runtime_error(
                    RUNTIME_VM_SHUTDOWN,
                    "Execution aborted: a concurrent task failed",
                    "A task spawned with `go` raised a runtime error. Fix the error in the spawned task.",
                    self.current_location,
                ));
            }

            if self.ip >= self.shared.chunk.code.len() {
                if self.current_task_id != 0 {
                    // Non-main task ran past end of code — complete it.
                    let result = self.stack.pop().unwrap_or(Value::Unit);
                    self.shared.store_task_result(self.current_task_id, result);
                    // Try to pick up another local task (cooperative yield).
                    if self.pick_next_task() {
                        continue;
                    }
                }
                return Ok(());
            }

            let op = self.shared.chunk.code[self.ip];
            let line = self.shared.chunk.location_at(self.ip).ok_or_else(|| {
                internal_error(
                    "Missing source location for instruction",
                    "This indicates a compiler/bytecode bug. Please report it.",
                    SourceLocation::new(1, 1),
                )
            })?;
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
                continue;
            }

            match op {
                Op::Return => {
                    let return_val = self.pop(line)?;
                    if let Some(frame) = self.call_stack.pop() {
                        self.stack.truncate(frame.base_slot);
                        self.push(return_val)?;
                        self.ip = frame.return_ip;
                    } else if self.current_task_id == 0 {
                        return Ok(());
                    } else {
                        self.shared
                            .store_task_result(self.current_task_id, return_val);
                        if !self.pick_next_task() {
                            return Ok(());
                        }
                    }
                }
                Op::Halt => return Ok(()),
                Op::Panic => {
                    let val = self.pop(line)?;
                    return Err(runtime_error(
                        RUNTIME_PROGRAM_PANIC,
                        format!("panic: {val}"),
                        "Remove the panic or guard the failing condition before calling panic.",
                        line,
                    ));
                }
                _ => {
                    return Err(internal_error(
                        format!("Unhandled opcode in VM dispatcher: {op:?}"),
                        "This indicates a VM dispatch bug. Please report it.",
                        line,
                    ));
                }
            }
        }
    }

    /// Try to pick up a task from the shared queue. Returns true if a task was loaded.
    fn pick_next_task(&mut self) -> bool {
        if let Some(task) = self.shared.try_dequeue_task() {
            self.load_task(task);
            true
        } else {
            false
        }
    }
}
