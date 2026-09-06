//! Deadline-bounded channel send and receive operations.

use std::time::{Duration, Instant};

use fpas_bytecode::{Register, Value};

use super::super::super::VmError;
use super::super::super::channels::{CANCELLATION_POLL_INTERVAL, ReceiveState, SendState};
use super::super::super::worker::Worker;
use super::super::TaskSuspension;
use super::{CLOSED_ERROR, error, ok};

const SEND_TIMEOUT_ERROR: &str = "Channel send timed out";
const RECEIVE_TIMEOUT_ERROR: &str = "Channel receive timed out";

impl Worker {
    pub(super) fn timeout_channel_send(
        &mut self,
        handle: u64,
        mut value: Value,
        timeout: Duration,
    ) -> Result<Value, VmError> {
        let started = Instant::now();
        let mut first_attempt = true;
        loop {
            if !first_attempt && remaining(timeout, started).is_none() {
                return Ok(error(SEND_TIMEOUT_ERROR));
            }
            first_attempt = false;
            match self
                .hosted
                .channels
                .send(handle, value, false, None)
                .map_err(|message| self.channel_runtime_error(message))?
            {
                SendState::Sent => return Ok(ok(Value::Boolean(true))),
                SendState::Closed => return Ok(error(CLOSED_ERROR)),
                SendState::Pending(pending) => value = pending,
                SendState::Cancelled => {
                    unreachable!("channel timeout does not observe cancellation")
                }
            }
            let Some(remaining) = remaining(timeout, started) else {
                return Ok(error(SEND_TIMEOUT_ERROR));
            };
            if self.help_one_channel_task()? {
                continue;
            }
            if self.channel_scheduler_stopped() {
                return Ok(error(CLOSED_ERROR));
            }
            match self
                .hosted
                .channels
                .send(
                    handle,
                    value,
                    false,
                    Some(remaining.min(CANCELLATION_POLL_INTERVAL)),
                )
                .map_err(|message| self.channel_runtime_error(message))?
            {
                SendState::Sent => return Ok(ok(Value::Boolean(true))),
                SendState::Closed => return Ok(error(CLOSED_ERROR)),
                SendState::Pending(pending) => value = pending,
                SendState::Cancelled => {
                    unreachable!("channel timeout does not observe cancellation")
                }
            }
        }
    }

    pub(super) fn timeout_channel_receive(
        &mut self,
        handle: u64,
        timeout: Duration,
    ) -> Result<Value, VmError> {
        let started = Instant::now();
        let mut first_attempt = true;
        loop {
            if !first_attempt && remaining(timeout, started).is_none() {
                return Ok(error(RECEIVE_TIMEOUT_ERROR));
            }
            first_attempt = false;
            match self
                .hosted
                .channels
                .receive(handle, false, None)
                .map_err(|message| self.channel_runtime_error(message))?
            {
                ReceiveState::Received(value) => return Ok(ok(value)),
                ReceiveState::Closed => return Ok(error(CLOSED_ERROR)),
                ReceiveState::Pending => {}
                ReceiveState::Cancelled => {
                    unreachable!("channel timeout does not observe cancellation")
                }
            }
            let Some(remaining) = remaining(timeout, started) else {
                return Ok(error(RECEIVE_TIMEOUT_ERROR));
            };
            if self.help_one_channel_task()? {
                continue;
            }
            if self.channel_scheduler_stopped() {
                return Ok(error(CLOSED_ERROR));
            }
            match self
                .hosted
                .channels
                .receive(
                    handle,
                    false,
                    Some(remaining.min(CANCELLATION_POLL_INTERVAL)),
                )
                .map_err(|message| self.channel_runtime_error(message))?
            {
                ReceiveState::Received(value) => return Ok(ok(value)),
                ReceiveState::Closed => return Ok(error(CLOSED_ERROR)),
                ReceiveState::Pending => {}
                ReceiveState::Cancelled => {
                    unreachable!("channel timeout does not observe cancellation")
                }
            }
        }
    }

    pub(super) fn debug_timeout_channel_send(
        &mut self,
        handle: u64,
        value: Value,
        timeout: Duration,
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        match self
            .hosted
            .channels
            .send(handle, value, false, None)
            .map_err(|message| self.channel_runtime_error(message))?
        {
            SendState::Sent => Ok(Some(Some(ok(Value::Boolean(true))))),
            SendState::Closed => Ok(Some(Some(error(CLOSED_ERROR)))),
            SendState::Pending(_) if timeout.is_zero() => Ok(Some(Some(error(SEND_TIMEOUT_ERROR)))),
            SendState::Pending(value) => {
                self.task_suspension = Some(TaskSuspension::ChannelSendTimeout {
                    handle,
                    value,
                    deadline_millis: self.channel_deadline(timeout),
                    destination,
                });
                self.suspend_requested = true;
                Ok(Some(None))
            }
            SendState::Cancelled => unreachable!("channel timeout does not observe cancellation"),
        }
    }

    pub(super) fn debug_timeout_channel_receive(
        &mut self,
        handle: u64,
        timeout: Duration,
        destination: Option<Register>,
    ) -> Result<Option<Option<Value>>, VmError> {
        match self
            .hosted
            .channels
            .receive(handle, false, None)
            .map_err(|message| self.channel_runtime_error(message))?
        {
            ReceiveState::Received(value) => Ok(Some(Some(ok(value)))),
            ReceiveState::Closed => Ok(Some(Some(error(CLOSED_ERROR)))),
            ReceiveState::Pending if timeout.is_zero() => {
                Ok(Some(Some(error(RECEIVE_TIMEOUT_ERROR))))
            }
            ReceiveState::Pending => {
                self.task_suspension = Some(TaskSuspension::ChannelReceiveTimeout {
                    handle,
                    deadline_millis: self.channel_deadline(timeout),
                    destination,
                });
                self.suspend_requested = true;
                Ok(Some(None))
            }
            ReceiveState::Cancelled => {
                unreachable!("channel timeout does not observe cancellation")
            }
        }
    }

    pub(in crate::vm::tasks) fn poll_debug_channel_send_timeout(
        &mut self,
        handle: u64,
        value: Value,
        deadline_millis: u64,
        destination: Option<Register>,
    ) -> Result<bool, VmError> {
        if self.channel_deadline_reached(deadline_millis) {
            return self.finish_debug_channel_poll(Some(error(SEND_TIMEOUT_ERROR)), destination);
        }
        let result = match self
            .hosted
            .channels
            .send(handle, value, false, None)
            .map_err(|message| self.channel_runtime_error(message))?
        {
            SendState::Sent => Some(ok(Value::Boolean(true))),
            SendState::Closed => Some(error(CLOSED_ERROR)),
            SendState::Pending(value) => {
                self.task_suspension = Some(TaskSuspension::ChannelSendTimeout {
                    handle,
                    value,
                    deadline_millis,
                    destination,
                });
                None
            }
            SendState::Cancelled => unreachable!("channel timeout does not observe cancellation"),
        };
        self.finish_debug_channel_poll(result, destination)
    }

    pub(in crate::vm::tasks) fn poll_debug_channel_receive_timeout(
        &mut self,
        handle: u64,
        deadline_millis: u64,
        destination: Option<Register>,
    ) -> Result<bool, VmError> {
        if self.channel_deadline_reached(deadline_millis) {
            return self.finish_debug_channel_poll(Some(error(RECEIVE_TIMEOUT_ERROR)), destination);
        }
        let result = match self
            .hosted
            .channels
            .receive(handle, false, None)
            .map_err(|message| self.channel_runtime_error(message))?
        {
            ReceiveState::Received(value) => Some(ok(value)),
            ReceiveState::Closed => Some(error(CLOSED_ERROR)),
            ReceiveState::Pending => {
                self.task_suspension = Some(TaskSuspension::ChannelReceiveTimeout {
                    handle,
                    deadline_millis,
                    destination,
                });
                None
            }
            ReceiveState::Cancelled => {
                unreachable!("channel timeout does not observe cancellation")
            }
        };
        self.finish_debug_channel_poll(result, destination)
    }

    pub(super) fn channel_timeout(&self, value: &Value) -> Result<Duration, VmError> {
        let Value::Integer(milliseconds) = value else {
            return Err(self.task_type_error("non-negative timeout in milliseconds", value));
        };
        let milliseconds = u64::try_from(*milliseconds)
            .map_err(|_| self.task_type_error("non-negative timeout in milliseconds", value))?;
        Ok(Duration::from_millis(milliseconds))
    }

    fn channel_deadline(&self, timeout: Duration) -> u64 {
        let timeout_millis = timeout.as_millis().min(u128::from(u64::MAX)) as u64;
        self.debug_clock_ref()
            .now_millis()
            .saturating_add(timeout_millis)
    }

    fn channel_deadline_reached(&self, deadline_millis: u64) -> bool {
        self.debug_clock_ref().now_millis() >= deadline_millis
    }
}

fn remaining(timeout: Duration, started: Instant) -> Option<Duration> {
    let remaining = timeout.saturating_sub(started.elapsed());
    (!remaining.is_zero()).then_some(remaining)
}
