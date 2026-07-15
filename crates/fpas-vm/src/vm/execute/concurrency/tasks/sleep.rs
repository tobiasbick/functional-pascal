//! Cooperative `Std.Time.Sleep` execution for spawned tasks.
//!
//! **Documentation:** `docs/pascal/std/host/time.md`.

use crate::vm::diagnostics::VmError;
use crate::vm::{Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{RUNTIME_NUMERIC_DOMAIN_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH};

impl Worker {
    /// Suspend the current spawned task until its sleep deadline.
    pub(in crate::vm::execute::concurrency) fn exec_task_sleep(
        &mut self,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        let value = self.pop(line)?;
        let Value::Integer(milliseconds) = value else {
            return Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("Sleep expected integer, got `{}`", value.type_name()),
                "Pass a non-negative integer number of milliseconds to `Std.Time.Sleep`.",
                line,
            ));
        };
        if milliseconds < 0 {
            return Err(runtime_error(
                RUNTIME_NUMERIC_DOMAIN_ERROR,
                format!("Sleep expects a non-negative millisecond count, got {milliseconds}"),
                "Pass `0` or a positive integer number of milliseconds.",
                line,
            ));
        }

        let mut task = self.save_task();
        if task.stack.is_empty() {
            task.stack.shrink_to_fit();
        }
        if task.call_stack.is_empty() {
            task.call_stack.shrink_to_fit();
        }
        self.shared.schedule_task_after(task, milliseconds as u64);
        self.task_suspended = true;
        Ok(())
    }
}
