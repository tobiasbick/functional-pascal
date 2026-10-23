//! Ordinary go task creation using the shared register-task entry layout and scope ownership.
//!
//! Documentation: `docs/pascal/language/concurrency/README.md`.

use super::TaskState;
use crate::vm::{VmError, diagnostics, worker::Worker};
use fpas_bytecode::{AbcOperands, Register, Value};
use fpas_diagnostics::codes::{RUNTIME_INVALID_TASK, RUNTIME_WRONG_CALL_ARITY};

impl Worker {
    /// Decode a go expression and enroll its retained child in the current routine's scope.
    pub(in crate::vm) fn spawn_task(&mut self, operands: AbcOperands) -> Result<(), VmError> {
        let scheduler = self
            .scheduler
            .as_ref()
            .ok_or_else(|| self.unavailable_opcode(fpas_bytecode::Opcode::SpawnTask))?
            .clone();
        let (callee_register, argument_base) = (operands.b, operands.c);
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
        self.require_transferable_values(std::slice::from_ref(&Value::Function(function.clone())))?;
        self.require_transferable_values(&arguments)?;
        let scope = self.spawning_scope().ok_or_else(|| {
            self.runtime_error(
                RUNTIME_INVALID_TASK,
                "Go requires an owning scope in the same routine",
                "Open `scope ... end scope;` around the start, or pass an explicit TaskGroup and use StartTaskInGroup.",
            )
        })?;
        let destination = if operands.a == u16::MAX {
            None
        } else {
            let destination = Register::new(operands.a).map_err(|error| {
                diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    error.to_string(),
                )
            })?;
            if self.base + usize::from(destination.get()) >= self.active_register_count() {
                return Err(diagnostics::internal(
                    self.executable.executable(),
                    self.current_address,
                    "Task destination is outside the active register window",
                ));
            }
            Some(destination)
        };
        let id = scheduler.alloc_id();
        let mut task = TaskState::entry_in_domain(
            id,
            &function,
            info,
            arguments,
            crate::vm::domain::related_domain(&self.domain),
        );
        task.owner_scope = scheduler.scopes.enroll(scope, id);
        scheduler.register_result(id);
        if let Some(destination) = destination {
            self.write(destination, Value::Task(id))?;
        }
        // A fresh entry has no execution roots to publish yet.
        scheduler.enqueue_published(task);
        Ok(())
    }
}

#[cfg(test)]
mod tests;
