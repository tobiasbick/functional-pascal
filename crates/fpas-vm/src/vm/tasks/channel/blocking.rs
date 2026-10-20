//! Blocking bounded-channel operations of the main task.
//!
//! The main task never runs queued tasks inline here: it may itself be the counterpart that a
//! queued task waits for, so executing that task on this thread could deadlock until the task's
//! own I/O times out. Pool workers make progress while the main task parks on the channel.

use fpas_bytecode::Value;

use super::super::super::VmError;
use super::super::super::channels::{CANCELLATION_POLL_INTERVAL, ReceiveState, SendState};
use super::super::super::worker::Worker;
use super::ok;
use crate::vm::tasks::errors::TaskErrorKind;

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
                SendState::Closed => {
                    return self.task_outcome_error(
                        TaskErrorKind::Closed,
                        if token.is_some() {
                            "Std.Tasks.SendWithCancellation"
                        } else {
                            "Std.Tasks.Send"
                        },
                    );
                }
                SendState::Cancelled => {
                    return self.task_outcome_error(
                        TaskErrorKind::Cancelled,
                        if token.is_some() {
                            "Std.Tasks.SendWithCancellation"
                        } else {
                            "Std.Tasks.Send"
                        },
                    );
                }
                SendState::Pending(pending) => value = pending,
            }
            if self.channel_scheduler_stopped() {
                return self.task_outcome_error(
                    TaskErrorKind::Closed,
                    if token.is_some() {
                        "Std.Tasks.SendWithCancellation"
                    } else {
                        "Std.Tasks.Send"
                    },
                );
            }
            self.check_task_cancellation()?;
            let cancelled = self.channel_cancelled(token)?;
            match self
                .hosted
                .channels
                .send(handle, value, cancelled, Some(CANCELLATION_POLL_INTERVAL))
                .map_err(|message| self.channel_runtime_error(message))?
            {
                SendState::Sent => return Ok(ok(Value::Boolean(true))),
                SendState::Closed => {
                    return self.task_outcome_error(
                        TaskErrorKind::Closed,
                        if token.is_some() {
                            "Std.Tasks.SendWithCancellation"
                        } else {
                            "Std.Tasks.Send"
                        },
                    );
                }
                SendState::Cancelled => {
                    return self.task_outcome_error(
                        TaskErrorKind::Cancelled,
                        if token.is_some() {
                            "Std.Tasks.SendWithCancellation"
                        } else {
                            "Std.Tasks.Send"
                        },
                    );
                }
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
                ReceiveState::Closed => {
                    return self.task_outcome_error(
                        TaskErrorKind::Closed,
                        if token.is_some() {
                            "Std.Tasks.ReceiveWithCancellation"
                        } else {
                            "Std.Tasks.Receive"
                        },
                    );
                }
                ReceiveState::Cancelled => {
                    return self.task_outcome_error(
                        TaskErrorKind::Cancelled,
                        if token.is_some() {
                            "Std.Tasks.ReceiveWithCancellation"
                        } else {
                            "Std.Tasks.Receive"
                        },
                    );
                }
                ReceiveState::Pending => {}
            }
            if self.channel_scheduler_stopped() {
                return self.task_outcome_error(
                    TaskErrorKind::Closed,
                    if token.is_some() {
                        "Std.Tasks.ReceiveWithCancellation"
                    } else {
                        "Std.Tasks.Receive"
                    },
                );
            }
            self.check_task_cancellation()?;
            let cancelled = self.channel_cancelled(token)?;
            match self
                .hosted
                .channels
                .receive(handle, cancelled, Some(CANCELLATION_POLL_INTERVAL))
                .map_err(|message| self.channel_runtime_error(message))?
            {
                ReceiveState::Received(value) => return Ok(ok(value)),
                ReceiveState::Closed => {
                    return self.task_outcome_error(
                        TaskErrorKind::Closed,
                        if token.is_some() {
                            "Std.Tasks.ReceiveWithCancellation"
                        } else {
                            "Std.Tasks.Receive"
                        },
                    );
                }
                ReceiveState::Cancelled => {
                    return self.task_outcome_error(
                        TaskErrorKind::Cancelled,
                        if token.is_some() {
                            "Std.Tasks.ReceiveWithCancellation"
                        } else {
                            "Std.Tasks.Receive"
                        },
                    );
                }
                ReceiveState::Pending => {}
            }
        }
    }
}
