//! FIFO storage and blocking coordination for typed FPAS channel handles.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::vm::shared::wakeups::{WakeRegistration, WakeSignal, WakeSource};
use fpas_bytecode::Value;

const HANDLE_TAG: u64 = 0x4348_0000_0000_0000;
const HANDLE_TAG_MASK: u64 = 0xFFFF_0000_0000_0000;
pub(in crate::vm) const MAX_CHANNEL_CAPACITY: usize = 1_048_576;
pub(in crate::vm) const CANCELLATION_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Outcome of one send attempt.
pub(in crate::vm) enum SendState {
    /// The value was appended to the channel.
    Sent,
    /// The channel is full and retains no ownership of the value.
    Pending(Value),
    /// The channel was closed before the value could be sent.
    Closed,
    /// Cancellation was observed before the value could be sent.
    Cancelled,
}

/// Outcome of one receive attempt.
pub(in crate::vm) enum ReceiveState {
    /// The oldest buffered value was removed.
    Received(Value),
    /// The open channel has no buffered value.
    Pending,
    /// The closed channel has no buffered value left.
    Closed,
    /// Cancellation was observed before a value became available.
    Cancelled,
}

struct ChannelState {
    values: VecDeque<Value>,
    capacity: usize,
    closed: bool,
}

struct Channel {
    state: Mutex<ChannelState>,
    can_send: Arc<WakeSource>,
    can_receive: Arc<WakeSource>,
}

/// Channel identities and state owned by one VM instance.
pub(in crate::vm) struct ChannelRegistry {
    next_handle: AtomicU64,
    entries: Mutex<HashMap<u64, Arc<Channel>>>,
}

impl ChannelRegistry {
    /// Validate an identity without observing or consuming a buffered value.
    pub(in crate::vm) fn validate(&self, handle: u64) -> Result<(), String> {
        self.channel(handle).map(|_| ())
    }

    /// Register before probing whether a send can commit.
    pub(in crate::vm) fn subscribe_send(
        &self,
        handle: u64,
        signal: &Arc<WakeSignal>,
    ) -> Result<WakeRegistration, String> {
        Ok(self.channel(handle)?.can_send.subscribe(signal))
    }

    /// Register before probing whether a receive can commit.
    pub(in crate::vm) fn subscribe_receive(
        &self,
        handle: u64,
        signal: &Arc<WakeSignal>,
    ) -> Result<WakeRegistration, String> {
        Ok(self.channel(handle)?.can_receive.subscribe(signal))
    }
    /// Create an empty channel registry.
    pub(in crate::vm) fn new() -> Self {
        Self {
            next_handle: AtomicU64::new(HANDLE_TAG | 1),
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Create a bounded channel.
    pub(in crate::vm) fn create(&self, capacity: i64) -> Result<u64, String> {
        let capacity = usize::try_from(capacity).map_err(|_| capacity_error(capacity))?;
        if !(1..=MAX_CHANNEL_CAPACITY).contains(&capacity) {
            return Err(capacity_error(i64::try_from(capacity).unwrap_or(i64::MAX)));
        }
        let handle = self.next_handle.fetch_add(1, Ordering::Relaxed);
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(
                handle,
                Arc::new(Channel {
                    state: Mutex::new(ChannelState {
                        values: VecDeque::with_capacity(capacity),
                        capacity,
                        closed: false,
                    }),
                    can_send: Arc::default(),
                    can_receive: Arc::default(),
                }),
            );
        Ok(handle)
    }

    /// Try to append a value, optionally waiting for one bounded interval.
    pub(in crate::vm) fn send(
        &self,
        handle: u64,
        value: Value,
        cancelled: bool,
        wait_for: Option<Duration>,
    ) -> Result<SendState, String> {
        let channel = self.channel(handle)?;
        let mut state = channel
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if cancelled {
            return Ok(SendState::Cancelled);
        }
        if state.closed {
            return Ok(SendState::Closed);
        }
        if state.values.len() < state.capacity {
            state.values.push_back(value);
            channel.can_receive.notify();
            return Ok(SendState::Sent);
        }
        if let Some(wait_for) = wait_for {
            let signal = WakeSignal::new();
            let _registration = channel.can_send.subscribe(&signal);
            drop(state);
            signal.wait(wait_for);
        }
        Ok(SendState::Pending(value))
    }

    /// Try to remove the oldest value, optionally waiting for one bounded interval.
    pub(in crate::vm) fn receive(
        &self,
        handle: u64,
        cancelled: bool,
        wait_for: Option<Duration>,
    ) -> Result<ReceiveState, String> {
        let channel = self.channel(handle)?;
        let mut state = channel
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if cancelled {
            return Ok(ReceiveState::Cancelled);
        }
        if let Some(value) = state.values.pop_front() {
            channel.can_send.notify();
            return Ok(ReceiveState::Received(value));
        }
        if state.closed {
            return Ok(ReceiveState::Closed);
        }
        if let Some(wait_for) = wait_for {
            let signal = WakeSignal::new();
            let _registration = channel.can_receive.subscribe(&signal);
            drop(state);
            signal.wait(wait_for);
        }
        Ok(ReceiveState::Pending)
    }

    /// Close a channel and wake every blocked sender and receiver.
    pub(in crate::vm) fn close(&self, handle: u64) -> Result<bool, String> {
        let channel = self.channel(handle)?;
        let mut state = channel
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let changed = !state.closed;
        state.closed = true;
        drop(state);
        channel.can_send.notify();
        channel.can_receive.notify();
        Ok(changed)
    }

    /// Close every channel during VM shutdown.
    pub(in crate::vm) fn shutdown(&self) {
        let channels = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for channel in channels {
            let mut state = channel
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.closed = true;
            drop(state);
            channel.can_send.notify();
            channel.can_receive.notify();
        }
    }

    fn channel(&self, handle: u64) -> Result<Arc<Channel>, String> {
        if handle & HANDLE_TAG_MASK != HANDLE_TAG {
            return Err("Value is not a channel handle".to_string());
        }
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&handle)
            .cloned()
            .ok_or_else(|| "Channel handle does not belong to this VM".to_string())
    }
}

fn capacity_error(capacity: i64) -> String {
    format!("Channel capacity must be in 1..={MAX_CHANNEL_CAPACITY}, got {capacity}")
}

#[cfg(test)]
mod tests;
