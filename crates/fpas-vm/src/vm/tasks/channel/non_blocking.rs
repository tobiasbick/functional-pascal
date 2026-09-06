//! Immediate bounded-channel operations.

use fpas_bytecode::Value;

use super::super::super::VmError;
use super::super::super::channels::{ReceiveState, SendState};
use super::super::super::worker::Worker;
use super::{CLOSED_ERROR, error, ok};

impl Worker {
    pub(super) fn try_channel_send(&self, handle: u64, value: Value) -> Result<Value, VmError> {
        match self
            .hosted
            .channels
            .send(handle, value, false, None)
            .map_err(|message| self.channel_runtime_error(message))?
        {
            SendState::Sent => Ok(ok(Value::Boolean(true))),
            SendState::Pending(_) => Ok(ok(Value::Boolean(false))),
            SendState::Closed => Ok(error(CLOSED_ERROR)),
            SendState::Cancelled => unreachable!("TrySend does not observe cancellation"),
        }
    }

    pub(super) fn try_channel_receive(&self, handle: u64) -> Result<Value, VmError> {
        match self
            .hosted
            .channels
            .receive(handle, false, None)
            .map_err(|message| self.channel_runtime_error(message))?
        {
            ReceiveState::Received(value) => Ok(ok(Value::option_some(value))),
            ReceiveState::Pending => Ok(ok(Value::OptionNone)),
            ReceiveState::Closed => Ok(error(CLOSED_ERROR)),
            ReceiveState::Cancelled => unreachable!("TryReceive does not observe cancellation"),
        }
    }
}
