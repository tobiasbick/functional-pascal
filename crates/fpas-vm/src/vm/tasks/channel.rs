//! `Std.Task` bounded-channel intrinsic dispatch.

mod blocking;
mod debug;
mod non_blocking;
mod timeout;

use std::sync::Arc;

use fpas_bytecode::{Intrinsic, Register, TaskIntrinsic, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::super::worker::Worker;
use super::super::{VmError, diagnostics};
use super::pool;

const CLOSED_ERROR: &str = "Channel is closed";
const SEND_CANCELLED_ERROR: &str = "Channel send was cancelled";
const RECEIVE_CANCELLED_ERROR: &str = "Channel receive was cancelled";

impl Worker {
    pub(super) fn channel_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        arguments: &[Value],
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Task(operation) = intrinsic else {
            return Ok(None);
        };
        match operation {
            TaskIntrinsic::CreateChannel => {
                self.require_channel_arguments(arguments, 1)?;
                let Value::Integer(capacity) = arguments[0] else {
                    return Err(self.task_type_error("integer", &arguments[0]));
                };
                let handle = self
                    .hosted
                    .channels
                    .create(capacity)
                    .map_err(|message| self.channel_runtime_error(message))?;
                Ok(Some(Some(Value::OpaqueHandle(handle))))
            }
            TaskIntrinsic::Send | TaskIntrinsic::SendWithCancellation => {
                let cancellable = operation == TaskIntrinsic::SendWithCancellation;
                self.require_channel_arguments(arguments, if cancellable { 3 } else { 2 })?;
                let handle = self.channel_handle(&arguments[0])?;
                let token = cancellable
                    .then(|| self.cancellation_token(&arguments[2]))
                    .transpose()?;
                if self.debug_tasks {
                    self.debug_channel_send(handle, arguments[1].clone(), token, destination)
                } else {
                    self.blocking_channel_send(handle, arguments[1].clone(), token)
                        .map(|value| Some(Some(value)))
                }
            }
            TaskIntrinsic::TrySend => {
                self.require_channel_arguments(arguments, 2)?;
                let handle = self.channel_handle(&arguments[0])?;
                self.try_channel_send(handle, arguments[1].clone())
                    .map(|value| Some(Some(value)))
            }
            TaskIntrinsic::SendWithTimeout => {
                self.require_channel_arguments(arguments, 3)?;
                let handle = self.channel_handle(&arguments[0])?;
                let timeout = self.channel_timeout(&arguments[2])?;
                if self.debug_tasks {
                    self.debug_timeout_channel_send(
                        handle,
                        arguments[1].clone(),
                        timeout,
                        destination,
                    )
                } else {
                    self.timeout_channel_send(handle, arguments[1].clone(), timeout)
                        .map(|value| Some(Some(value)))
                }
            }
            TaskIntrinsic::Receive | TaskIntrinsic::ReceiveWithCancellation => {
                let cancellable = operation == TaskIntrinsic::ReceiveWithCancellation;
                self.require_channel_arguments(arguments, if cancellable { 2 } else { 1 })?;
                let handle = self.channel_handle(&arguments[0])?;
                let token = cancellable
                    .then(|| self.cancellation_token(&arguments[1]))
                    .transpose()?;
                if self.debug_tasks {
                    self.debug_channel_receive(handle, token, destination)
                } else {
                    self.blocking_channel_receive(handle, token)
                        .map(|value| Some(Some(value)))
                }
            }
            TaskIntrinsic::TryReceive => {
                self.require_channel_arguments(arguments, 1)?;
                let handle = self.channel_handle(&arguments[0])?;
                self.try_channel_receive(handle)
                    .map(|value| Some(Some(value)))
            }
            TaskIntrinsic::ReceiveWithTimeout => {
                self.require_channel_arguments(arguments, 2)?;
                let handle = self.channel_handle(&arguments[0])?;
                let timeout = self.channel_timeout(&arguments[1])?;
                if self.debug_tasks {
                    self.debug_timeout_channel_receive(handle, timeout, destination)
                } else {
                    self.timeout_channel_receive(handle, timeout)
                        .map(|value| Some(Some(value)))
                }
            }
            TaskIntrinsic::CloseChannel => {
                self.require_channel_arguments(arguments, 1)?;
                let handle = self.channel_handle(&arguments[0])?;
                let changed = self
                    .hosted
                    .channels
                    .close(handle)
                    .map_err(|message| self.channel_runtime_error(message))?;
                Ok(Some(Some(Value::Boolean(changed))))
            }
            TaskIntrinsic::CreateCancellationSource
            | TaskIntrinsic::GetCancellationToken
            | TaskIntrinsic::Cancel
            | TaskIntrinsic::IsCancellationRequested
            | TaskIntrinsic::Wait
            | TaskIntrinsic::WaitAll => Ok(None),
        }
    }

    pub(super) fn help_one_channel_task(&mut self) -> Result<bool, VmError> {
        let Some(scheduler) = self.scheduler.clone() else {
            return Ok(false);
        };
        let Some(task) = scheduler.try_dequeue() else {
            return Ok(false);
        };
        pool::run_helped(self, task, Arc::clone(&scheduler))?;
        Ok(true)
    }

    pub(super) fn channel_scheduler_stopped(&self) -> bool {
        self.scheduler
            .as_ref()
            .is_some_and(|scheduler| scheduler.is_shutdown())
    }

    fn channel_cancelled(&self, token: Option<u64>) -> Result<bool, VmError> {
        token.map_or(Ok(false), |token| {
            self.hosted
                .cancellations
                .is_cancelled(token)
                .map_err(|message| self.channel_runtime_error(message))
        })
    }

    fn cancellation_token(&self, value: &Value) -> Result<u64, VmError> {
        let Value::OpaqueHandle(token) = value else {
            return Err(self.task_type_error("CancellationToken", value));
        };
        self.hosted
            .cancellations
            .is_cancelled(*token)
            .map_err(|message| self.channel_runtime_error(message))?;
        Ok(*token)
    }

    fn channel_handle(&self, value: &Value) -> Result<u64, VmError> {
        match value {
            Value::OpaqueHandle(handle) => Ok(*handle),
            actual => Err(self.task_type_error("channel", actual)),
        }
    }

    fn require_channel_arguments(
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
                "Std.Task channel intrinsic expected {expected} arguments, got {}",
                arguments.len()
            ),
            "Check the compiler intrinsic signature and register argument count.",
        ))
    }

    fn channel_runtime_error(&self, message: String) -> VmError {
        diagnostics::at_address(
            self.executable.executable(),
            self.current_address,
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            message,
            "Pass a channel or cancellation handle created by this VM and a supported capacity.",
        )
    }
}

fn ok(value: Value) -> Value {
    Value::result_ok(value)
}

fn error(message: &str) -> Value {
    Value::result_error(Value::Str(message.into()))
}
