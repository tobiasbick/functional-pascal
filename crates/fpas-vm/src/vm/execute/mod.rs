mod aggregates;
mod concurrency;
mod control_calls;
mod enums;
mod io;
mod numeric;
mod result_option;
mod stack_scope;

use super::{Vm, VmError, internal_error, runtime_error};
use fpas_bytecode::{Op, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC;

impl Vm {
    pub fn run(&mut self) -> Result<(), VmError> {
        loop {
            if self.ip >= self.chunk.code.len() {
                if self.current_task_id != 0 {
                    self.complete_current_task(Value::Unit);
                    if self.schedule_next_task() {
                        continue;
                    }
                }
                return Ok(());
            }

            let op = self.chunk.code[self.ip];
            let line = self.chunk.location_at(self.ip).ok_or_else(|| {
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
                        self.complete_current_task(return_val);
                        if !self.schedule_next_task() {
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
}
