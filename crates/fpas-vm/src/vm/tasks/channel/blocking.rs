//! Blocking bounded-channel operations.

use fpas_bytecode::Value;

use super::super::super::VmError;
use super::super::super::channels::{CANCELLATION_POLL_INTERVAL, ReceiveState, SendState};
use super::super::super::worker::Worker;
use super::{CLOSED_ERROR, RECEIVE_CANCELLED_ERROR, SEND_CANCELLED_ERROR, error, ok};

impl Worker {
    pub(super) fn blocking_channel_send(
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
                .send(handle, value, cancelled, None)
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
                .send(handle, value, cancelled, Some(CANCELLATION_POLL_INTERVAL))
                .map_err(|message| self.channel_runtime_error(message))?
            {
                SendState::Sent => return Ok(ok(Value::Boolean(true))),
                SendState::Closed => return Ok(error(CLOSED_ERROR)),
                SendState::Cancelled => return Ok(error(SEND_CANCELLED_ERROR)),
                SendState::Pending(pending) => value = pending,
            }
        }
    }

    pub(super) fn blocking_channel_receive(
        &mut self,
        handle: u64,
        token: Option<u64>,
    ) -> Result<Value, VmError> {
        loop {
            let cancelled = self.channel_cancelled(token)?;
            match self
                .hosted
                .channels
                .receive(handle, cancelled, None)
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
                .receive(handle, cancelled, Some(CANCELLATION_POLL_INTERVAL))
                .map_err(|message| self.channel_runtime_error(message))?
            {
                ReceiveState::Received(value) => return Ok(ok(value)),
                ReceiveState::Closed => return Ok(error(CLOSED_ERROR)),
                ReceiveState::Cancelled => return Ok(error(RECEIVE_CANCELLED_ERROR)),
                ReceiveState::Pending => {}
            }
        }
    }
}
