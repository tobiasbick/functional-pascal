//! Ordinary go task creation using the shared register-task entry layout.
//!
//! Documentation: `docs/pascal/language/concurrency/README.md`.

use super::TaskState;
use crate::vm::{VmError, diagnostics, worker::Worker};
use fpas_bytecode::{AbcOperands, Register, Value};
use fpas_diagnostics::codes::{RUNTIME_INVALID_TASK, RUNTIME_WRONG_CALL_ARITY};

impl Worker {
    /// Decode ordinary go operands and publish a retained or detached task.
    pub(in crate::vm) fn spawn_task(
        &mut self,
        operands: AbcOperands,
        detached: bool,
    ) -> Result<(), VmError> {
        let scheduler = self
            .scheduler
            .as_ref()
            .ok_or_else(|| {
                self.unavailable_opcode(if detached {
                    fpas_bytecode::Opcode::SpawnDetachedTask
                } else {
                    fpas_bytecode::Opcode::SpawnTask
                })
            })?
            .clone();
        let (callee_register, argument_base) = if detached {
            (operands.a, operands.b)
        } else {
            (operands.b, operands.c)
        };
        let callee = self
            .read(Register::new(callee_register).map_err(|e| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    e.to_string(),
                )
            })?)?
            .clone();
        let Value::Function(function) = callee else {
            return Err(self.task_type_error("function", &callee));
        };
        if function.task_bound {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                self.current_address,
                RUNTIME_INVALID_TASK,
                format!(
                    "Cannot spawn task-bound closure `{}` across a task boundary",
                    function.name
                ),
                "Mutable captures make a closure task-bound. Pass immutable values instead, or invoke the closure on the same task.",
            ));
        }
        let target = function.function;
        let info = self
            .executable
            .executable()
            .functions
            .get(usize::from(target.get()))
            .ok_or_else(|| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Task target is outside the function table",
                )
            })?;
        let visible_arity = usize::from(info.arity)
            .checked_sub(usize::from(function.bound_receiver.is_some()))
            .ok_or_else(|| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Bound task target has no receiver parameter",
                )
            })?;
        if visible_arity != usize::from(operands.auxiliary) {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                self.current_address,
                RUNTIME_WRONG_CALL_ARITY,
                format!(
                    "Function `{}` expects {} arguments, got {}",
                    function.name, visible_arity, operands.auxiliary
                ),
                "Spawn the task with the declared number of arguments.",
            ));
        }
        let arguments = self.clone_window(argument_base, operands.auxiliary)?;
        let id = scheduler.alloc_id();
        let task = TaskState::entry(id, &function, info, arguments, !detached);
        if !detached {
            scheduler.register_result(id);
            self.write(
                Register::new(operands.a).map_err(|e| {
                    diagnostics::internal(
                        self.executable.executable(),
                        self.current_address,
                        e.to_string(),
                    )
                })?,
                Value::Task(id),
            )?;
        }
        scheduler.enqueue(task);
        Ok(())
    }
}
