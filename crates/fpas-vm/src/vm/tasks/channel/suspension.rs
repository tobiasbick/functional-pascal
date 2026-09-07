//! Cooperative suspension and polling for bounded-channel operations.

use fpas_bytecode::{Register, Value};

use super::super::super::VmError;
use super::super::super::channels::{ReceiveState, SendState};
use super::super::super::worker::Worker;
use super::super::TaskSuspension;
use super::{CLOSED_ERROR, RECEIVE_CANCELLED_ERROR, SEND_CANCELLED_ERROR, error, ok};

impl Worker {
    /// Send immediately or retain the value and destination until readiness changes.
    pub(super) fn start_channel_send(
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
            .send(handle, value, cancelled, None)
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

    /// Receive immediately or retain the destination until readiness changes.
    pub(super) fn start_channel_receive(
        &mut self,
        handle: u64,
        token: Option<u64>,
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        let cancelled = self.channel_cancelled(token)?;
        match self
            .hosted
            .channels
            .receive(handle, cancelled, None)
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

    /// Resume a pending send, observing closure and cancellation on every probe.
    pub(in crate::vm::tasks) fn poll_channel_send(
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
            .send(handle, value, cancelled, None)
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
        self.finish_channel_poll(result, destination)
    }

    /// Resume a pending receive, observing closure and cancellation on every probe.
    pub(in crate::vm::tasks) fn poll_channel_receive(
        &mut self,
        handle: u64,
        token: Option<u64>,
        destination: Option<Register>,
    ) -> Result<bool, VmError> {
        let cancelled = self.channel_cancelled(token)?;
        let result = match self
            .hosted
            .channels
            .receive(handle, cancelled, None)
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
        self.finish_channel_poll(result, destination)
    }

    /// Deliver a completed channel operation to its saved destination register.
    pub(super) fn finish_channel_poll(
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
}
