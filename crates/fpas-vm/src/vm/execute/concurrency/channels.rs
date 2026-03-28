use super::super::super::{ChannelState, Vm, VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_CHANNEL_CLOSED, RUNTIME_INVALID_CHANNEL, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};
use std::collections::VecDeque;

impl Vm {
    pub(super) fn exec_channel_make(
        &mut self,
        capacity: usize,
        _line: SourceLocation,
    ) -> Result<(), VmError> {
        let id = self.next_channel_id;
        self.next_channel_id += 1;
        self.channels.insert(
            id,
            ChannelState {
                buffer: VecDeque::new(),
                capacity,
                closed: false,
            },
        );
        self.push(Value::Channel(id))
    }

    pub(super) fn exec_channel_send(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let value = self.pop(line)?;
        let channel_id = self.pop_channel_id(line)?;

        let channel = self.get_channel_mut(channel_id, line)?;
        if channel.closed {
            return Err(runtime_error(
                RUNTIME_CHANNEL_CLOSED,
                "Cannot send on a closed channel",
                "Check `Std.Channel.Close` usage: do not send after closing.",
                line,
            ));
        }

        if channel.buffer.len() < channel.capacity {
            channel.buffer.push_back(value);
        } else {
            self.push(Value::Channel(channel_id))?;
            self.push(value)?;
            self.ip -= 1;
            self.exec_yield();
        }
        Ok(())
    }

    pub(super) fn exec_channel_recv(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let channel_id = self.pop_channel_id(line)?;
        let channel = self.get_channel_mut(channel_id, line)?;

        if let Some(value) = channel.buffer.pop_front() {
            self.push(value)?;
        } else if channel.closed {
            self.push(Value::Unit)?;
        } else {
            self.push(Value::Channel(channel_id))?;
            self.ip -= 1;
            self.exec_yield();
        }
        Ok(())
    }

    pub(super) fn exec_channel_try_recv(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let channel_id = self.pop_channel_id(line)?;
        let channel = self.get_channel_mut(channel_id, line)?;

        if let Some(value) = channel.buffer.pop_front() {
            self.push(Value::OptionSome(Box::new(value)))?;
        } else {
            self.push(Value::OptionNone)?;
        }
        Ok(())
    }

    pub(super) fn exec_channel_close(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let channel_id = self.pop_channel_id(line)?;
        let channel = self.get_channel_mut(channel_id, line)?;
        channel.closed = true;
        Ok(())
    }

    fn pop_channel_id(&mut self, line: SourceLocation) -> Result<u64, VmError> {
        let value = self.pop(line)?;
        match value {
            Value::Channel(id) => Ok(id),
            other => Err(runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!("Expected channel, got `{}`", other.type_name()),
                "Pass a channel created with `Std.Channel.Make`.",
                line,
            )),
        }
    }

    fn get_channel_mut(
        &mut self,
        id: u64,
        line: SourceLocation,
    ) -> Result<&mut ChannelState, VmError> {
        self.channels.get_mut(&id).ok_or_else(|| {
            runtime_error(
                RUNTIME_INVALID_CHANNEL,
                format!("Channel {id} does not exist"),
                "The channel may have been garbage-collected or was never created.",
                line,
            )
        })
    }
}
