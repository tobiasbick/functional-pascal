//! `Std.Task` bounded-channel intrinsic dispatch.

use std::sync::Arc;

use fpas_bytecode::{Intrinsic, Register, TaskIntrinsic, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INTRINSIC_STACK_STATE_ERROR, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::super::channels::{ReceiveState, SendState};
use super::super::worker::Worker;
use super::super::{VmError, diagnostics};
use super::{TaskSuspension, pool};

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

    fn blocking_channel_send(
        &mut self,
        handle: u64,
        mut value: Value,
        token: Option<u64>,
    ) -> Result<Value, VmError> {
        loop {
            let cancelled = self.channel_cancelled(token)?;
            match self
                .hosted
                .channels
                .send(handle, value, cancelled, false)
                .map_err(|message| self.channel_runtime_error(message))?
            {
                SendState::Sent => return Ok(ok(Value::Boolean(true))),
                SendState::Closed => return Ok(error(CLOSED_ERROR)),
                SendState::Cancelled => return Ok(error(SEND_CANCELLED_ERROR)),
                SendState::Pending(pending) => value = pending,
            }
            if self.help_one_channel_task()? {
                continue;
            }
            if self.channel_scheduler_stopped() {
                return Ok(error(CLOSED_ERROR));
            }
            let cancelled = self.channel_cancelled(token)?;
            match self
                .hosted
                .channels
                .send(handle, value, cancelled, true)
                .map_err(|message| self.channel_runtime_error(message))?
            {
                SendState::Sent => return Ok(ok(Value::Boolean(true))),
                SendState::Closed => return Ok(error(CLOSED_ERROR)),
                SendState::Cancelled => return Ok(error(SEND_CANCELLED_ERROR)),
                SendState::Pending(pending) => value = pending,
            }
        }
    }

    fn blocking_channel_receive(
        &mut self,
        handle: u64,
        token: Option<u64>,
    ) -> Result<Value, VmError> {
        loop {
            let cancelled = self.channel_cancelled(token)?;
            match self
                .hosted
                .channels
                .receive(handle, cancelled, false)
                .map_err(|message| self.channel_runtime_error(message))?
            {
                ReceiveState::Received(value) => return Ok(ok(value)),
                ReceiveState::Closed => return Ok(error(CLOSED_ERROR)),
                ReceiveState::Cancelled => return Ok(error(RECEIVE_CANCELLED_ERROR)),
                ReceiveState::Pending => {}
            }
            if self.help_one_channel_task()? {
                continue;
            }
            if self.channel_scheduler_stopped() {
                return Ok(error(CLOSED_ERROR));
            }
            let cancelled = self.channel_cancelled(token)?;
            match self
                .hosted
                .channels
                .receive(handle, cancelled, true)
                .map_err(|message| self.channel_runtime_error(message))?
            {
                ReceiveState::Received(value) => return Ok(ok(value)),
                ReceiveState::Closed => return Ok(error(CLOSED_ERROR)),
                ReceiveState::Cancelled => return Ok(error(RECEIVE_CANCELLED_ERROR)),
                ReceiveState::Pending => {}
            }
        }
    }

    fn debug_channel_send(
        &mut self,
        handle: u64,
        value: Value,
        token: Option<u64>,
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        let cancelled = self.channel_cancelled(token)?;
        match self
            .hosted
            .channels
            .send(handle, value, cancelled, false)
            .map_err(|message| self.channel_runtime_error(message))?
        {
            SendState::Sent => Ok(Some(Some(ok(Value::Boolean(true))))),
            SendState::Closed => Ok(Some(Some(error(CLOSED_ERROR)))),
            SendState::Cancelled => Ok(Some(Some(error(SEND_CANCELLED_ERROR)))),
            SendState::Pending(value) => {
                self.task_suspension = Some(TaskSuspension::ChannelSend {
                    handle,
                    value,
                    token,
                    destination,
                });
                self.suspend_requested = true;
                Ok(Some(None))
            }
        }
    }

    fn debug_channel_receive(
        &mut self,
        handle: u64,
        token: Option<u64>,
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        let cancelled = self.channel_cancelled(token)?;
        match self
            .hosted
            .channels
            .receive(handle, cancelled, false)
            .map_err(|message| self.channel_runtime_error(message))?
        {
            ReceiveState::Received(value) => Ok(Some(Some(ok(value)))),
            ReceiveState::Closed => Ok(Some(Some(error(CLOSED_ERROR)))),
            ReceiveState::Cancelled => Ok(Some(Some(error(RECEIVE_CANCELLED_ERROR)))),
            ReceiveState::Pending => {
                self.task_suspension = Some(TaskSuspension::ChannelReceive {
                    handle,
                    token,
                    destination,
                });
                self.suspend_requested = true;
                Ok(Some(None))
            }
        }
    }

    pub(super) fn poll_debug_channel_send(
        &mut self,
        handle: u64,
        value: Value,
        token: Option<u64>,
        destination: Option<Register>,
    ) -> Result<bool, VmError> {
        let cancelled = self.channel_cancelled(token)?;
        let result = match self
            .hosted
            .channels
            .send(handle, value, cancelled, false)
            .map_err(|message| self.channel_runtime_error(message))?
        {
            SendState::Sent => Some(ok(Value::Boolean(true))),
            SendState::Closed => Some(error(CLOSED_ERROR)),
            SendState::Cancelled => Some(error(SEND_CANCELLED_ERROR)),
            SendState::Pending(value) => {
                self.task_suspension = Some(TaskSuspension::ChannelSend {
                    handle,
                    value,
                    token,
                    destination,
                });
                None
            }
        };
        self.finish_debug_channel_poll(result, destination)
    }

    pub(super) fn poll_debug_channel_receive(
        &mut self,
        handle: u64,
        token: Option<u64>,
        destination: Option<Register>,
    ) -> Result<bool, VmError> {
        let cancelled = self.channel_cancelled(token)?;
        let result = match self
            .hosted
            .channels
            .receive(handle, cancelled, false)
            .map_err(|message| self.channel_runtime_error(message))?
        {
            ReceiveState::Received(value) => Some(ok(value)),
            ReceiveState::Closed => Some(error(CLOSED_ERROR)),
            ReceiveState::Cancelled => Some(error(RECEIVE_CANCELLED_ERROR)),
            ReceiveState::Pending => {
                self.task_suspension = Some(TaskSuspension::ChannelReceive {
                    handle,
                    token,
                    destination,
                });
                None
            }
        };
        self.finish_debug_channel_poll(result, destination)
    }

    fn finish_debug_channel_poll(
        &mut self,
        result: Option<Value>,
        destination: Option<Register>,
    ) -> Result<bool, VmError> {
        let Some(value) = result else {
            return Ok(false);
        };
        if let Some(destination) = destination {
            self.write(destination, value)?;
        }
        Ok(true)
    }

    fn help_one_channel_task(&mut self) -> Result<bool, VmError> {
        let Some(scheduler) = self.scheduler.clone() else {
            return Ok(false);
        };
        let Some(task) = scheduler.try_dequeue() else {
            return Ok(false);
        };
        pool::run_helped(self, task, Arc::clone(&scheduler))?;
        Ok(true)
    }

    fn channel_scheduler_stopped(&self) -> bool {
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
