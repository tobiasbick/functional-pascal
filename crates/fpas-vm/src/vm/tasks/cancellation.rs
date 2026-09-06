//! `Std.Task` cooperative-cancellation intrinsic dispatch.

use fpas_bytecode::{Intrinsic, TaskIntrinsic, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::super::worker::Worker;
use super::super::{VmError, diagnostics};

impl Worker {
    pub(super) fn cancellation_intrinsic(
        &self,
        intrinsic: Intrinsic,
        arguments: &[Value],
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Task(operation) = intrinsic else {
            return Ok(None);
        };
        let value = match operation {
            TaskIntrinsic::CreateCancellationSource => {
                self.require_cancellation_arguments(arguments, 0)?;
                Value::OpaqueHandle(self.hosted.cancellations.create_source())
            }
            TaskIntrinsic::GetCancellationToken => {
                self.require_cancellation_arguments(arguments, 1)?;
                let source = self.cancellation_handle(&arguments[0], "CancellationSource")?;
                Value::OpaqueHandle(
                    self.hosted
                        .cancellations
                        .token(source)
                        .map_err(|message| self.cancellation_error(message))?,
                )
            }
            TaskIntrinsic::Cancel => {
                self.require_cancellation_arguments(arguments, 1)?;
                let source = self.cancellation_handle(&arguments[0], "CancellationSource")?;
                Value::Boolean(
                    self.hosted
                        .cancellations
                        .cancel(source)
                        .map_err(|message| self.cancellation_error(message))?,
                )
            }
            TaskIntrinsic::IsCancellationRequested => {
                self.require_cancellation_arguments(arguments, 1)?;
                let token = self.cancellation_handle(&arguments[0], "CancellationToken")?;
                Value::Boolean(
                    self.hosted
                        .cancellations
                        .is_cancelled(token)
                        .map_err(|message| self.cancellation_error(message))?,
                )
            }
            TaskIntrinsic::Wait | TaskIntrinsic::WaitAll => return Ok(None),
        };
        Ok(Some(Some(value)))
    }

    fn require_cancellation_arguments(
        &self,
        arguments: &[Value],
        expected: usize,
    ) -> Result<(), VmError> {
        if arguments.len() == expected {
            return Ok(());
        }
        Err(self.runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            format!(
                "Std.Task cancellation intrinsic expected {expected} arguments, got {}",
                arguments.len()
            ),
            "Check the compiler intrinsic signature and register argument count.",
        ))
    }

    fn cancellation_handle(&self, value: &Value, expected: &str) -> Result<u64, VmError> {
        match value {
            Value::OpaqueHandle(handle) => Ok(*handle),
            actual => Err(self.task_type_error(expected, actual)),
        }
    }

    fn cancellation_error(&self, message: String) -> VmError {
        diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            message,
            "Pass a cancellation source or token created by this VM.",
        )
    }
}
