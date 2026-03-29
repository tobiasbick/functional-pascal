use super::super::super::diagnostics::VmError;
use super::super::super::{SharedChannel, Worker, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::{
    RUNTIME_CHANNEL_CLOSED, RUNTIME_INVALID_CHANNEL, RUNTIME_NUMERIC_DOMAIN_ERROR,
    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};
use std::sync::atomic::Ordering;

impl Worker {
    pub(super) fn exec_channel_make(
        &mut self,
        capacity: usize,
        _line: SourceLocation,
    ) -> Result<(), VmError> {
        let id = self.shared.alloc_channel_id();
        let (sender, receiver) = crossbeam_channel::bounded(capacity);
        let channel = SharedChannel {
            sender,
            receiver,
            closed: std::sync::atomic::AtomicBool::new(false),
        };
        self.shared
            .channels
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(id, channel);
        self.push(Value::Channel(id))
    }

    pub(super) fn exec_channel_make_buffered(
        &mut self,
        capacity: i64,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        if capacity < 0 {
            return Err(runtime_error(
                RUNTIME_NUMERIC_DOMAIN_ERROR,
                "Channel buffer size cannot be negative",
                "Pass `0` or a positive integer to `Std.Channel.MakeBuffered`.",
                line,
            ));
        }

        self.exec_channel_make(capacity as usize, line)
    }

    pub(super) fn exec_channel_send(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let value = self.pop(line)?;
        let channel_id = self.pop_channel_id(line)?;

        let channels = self
            .shared
            .channels
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let channel = channels.get(&channel_id).ok_or_else(|| {
            runtime_error(
                RUNTIME_INVALID_CHANNEL,
                format!("Channel {channel_id} does not exist"),
                "The channel may have been garbage-collected or was never created.",
                line,
            )
        })?;

        if channel.closed.load(Ordering::Acquire) {
            return Err(runtime_error(
                RUNTIME_CHANNEL_CLOSED,
                "Cannot send on a closed channel",
                "Check `Std.Channel.Close` usage: do not send after closing.",
                line,
            ));
        }

        match channel.sender.try_send(value.clone()) {
            Ok(()) => Ok(()),
            Err(crossbeam_channel::TrySendError::Full(_)) => {
                drop(channels);
                // Buffer full — re-push args and yield for retry.
                self.push(Value::Channel(channel_id))?;
                self.push(value)?;
                self.ip -= 1;
                self.exec_yield();
                Ok(())
            }
            Err(crossbeam_channel::TrySendError::Disconnected(_)) => Err(runtime_error(
                RUNTIME_CHANNEL_CLOSED,
                "Cannot send on a disconnected channel",
                "The receiving end of the channel has been dropped.",
                line,
            )),
        }
    }

    pub(super) fn exec_channel_recv(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let channel_id = self.pop_channel_id(line)?;

        let channels = self
            .shared
            .channels
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let channel = channels.get(&channel_id).ok_or_else(|| {
            runtime_error(
                RUNTIME_INVALID_CHANNEL,
                format!("Channel {channel_id} does not exist"),
                "The channel may have been garbage-collected or was never created.",
                line,
            )
        })?;

        match channel.receiver.try_recv() {
            Ok(value) => {
                drop(channels);
                self.push(value)?;
            }
            Err(crossbeam_channel::TryRecvError::Empty) => {
                if channel.closed.load(Ordering::Acquire) {
                    return Err(runtime_error(
                        RUNTIME_CHANNEL_CLOSED,
                        "Cannot receive from a closed, empty channel",
                        "Use `Std.Channel.TryReceive` if the channel may already be closed and empty.",
                        line,
                    ));
                } else {
                    drop(channels);
                    // Channel empty — re-push and yield for retry.
                    self.push(Value::Channel(channel_id))?;
                    self.ip -= 1;
                    self.exec_yield();
                }
            }
            Err(crossbeam_channel::TryRecvError::Disconnected) => Err(runtime_error(
                RUNTIME_CHANNEL_CLOSED,
                "Cannot receive from a disconnected channel",
                "Use `Std.Channel.TryReceive` or keep at least one sender alive until the final value is sent.",
                line,
            ))?,
        }
        Ok(())
    }

    pub(super) fn exec_channel_try_recv(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let channel_id = self.pop_channel_id(line)?;

        let channels = self
            .shared
            .channels
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let channel = channels.get(&channel_id).ok_or_else(|| {
            runtime_error(
                RUNTIME_INVALID_CHANNEL,
                format!("Channel {channel_id} does not exist"),
                "The channel may have been garbage-collected or was never created.",
                line,
            )
        })?;

        match channel.receiver.try_recv() {
            Ok(value) => {
                drop(channels);
                self.push(Value::OptionSome(Box::new(value)))?;
            }
            Err(_) => {
                drop(channels);
                self.push(Value::OptionNone)?;
            }
        }
        Ok(())
    }

    pub(super) fn exec_channel_close(&mut self, line: SourceLocation) -> Result<(), VmError> {
        let channel_id = self.pop_channel_id(line)?;
        let channels = self
            .shared
            .channels
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let channel = channels.get(&channel_id).ok_or_else(|| {
            runtime_error(
                RUNTIME_INVALID_CHANNEL,
                format!("Channel {channel_id} does not exist"),
                "The channel may have been garbage-collected or was never created.",
                line,
            )
        })?;
        channel.closed.store(true, Ordering::Release);
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
}
