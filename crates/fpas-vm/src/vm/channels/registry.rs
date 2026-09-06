//! FIFO storage and blocking coordination for typed FPAS channel handles.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

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
    can_send: Condvar,
    can_receive: Condvar,
}

/// Channel identities and state owned by one VM instance.
pub(in crate::vm) struct ChannelRegistry {
    next_handle: AtomicU64,
    entries: Mutex<HashMap<u64, Arc<Channel>>>,
}

impl ChannelRegistry {
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
                    can_send: Condvar::new(),
                    can_receive: Condvar::new(),
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
            channel.can_receive.notify_one();
            return Ok(SendState::Sent);
        }
        if let Some(wait_for) = wait_for {
            let (guard, _) = channel
                .can_send
                .wait_timeout(state, wait_for)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            drop(guard);
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
            channel.can_send.notify_one();
            return Ok(ReceiveState::Received(value));
        }
        if state.closed {
            return Ok(ReceiveState::Closed);
        }
        if let Some(wait_for) = wait_for {
            let (guard, _) = channel
                .can_receive
                .wait_timeout(state, wait_for)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            drop(guard);
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
        channel.can_send.notify_all();
        channel.can_receive.notify_all();
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
            channel.can_send.notify_all();
            channel.can_receive.notify_all();
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
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use super::{CANCELLATION_POLL_INTERVAL, ChannelRegistry, ReceiveState, SendState};
    use fpas_bytecode::Value;

    #[test]
    fn preserves_fifo_order_and_reports_full_capacity() {
        let registry = ChannelRegistry::new();
        let handle = registry.create(2).expect("channel");
        assert!(matches!(
            registry.send(handle, Value::Integer(1), false, None),
            Ok(SendState::Sent)
        ));
        assert!(matches!(
            registry.send(handle, Value::Integer(2), false, None),
            Ok(SendState::Sent)
        ));
        assert!(matches!(
            registry.send(handle, Value::Integer(3), false, None),
            Ok(SendState::Pending(Value::Integer(3)))
        ));
        assert!(matches!(
            registry.receive(handle, false, None),
            Ok(ReceiveState::Received(Value::Integer(1)))
        ));
        assert!(matches!(
            registry.receive(handle, false, None),
            Ok(ReceiveState::Received(Value::Integer(2)))
        ));
    }

    #[test]
    fn close_is_idempotent_and_drains_buffer_before_closed() {
        let registry = ChannelRegistry::new();
        let handle = registry.create(1).expect("channel");
        assert!(matches!(
            registry.send(handle, Value::Integer(7), false, None),
            Ok(SendState::Sent)
        ));
        assert_eq!(registry.close(handle), Ok(true));
        assert_eq!(registry.close(handle), Ok(false));
        assert!(matches!(
            registry.receive(handle, false, None),
            Ok(ReceiveState::Received(Value::Integer(7)))
        ));
        assert!(matches!(
            registry.receive(handle, false, None),
            Ok(ReceiveState::Closed)
        ));
    }

    #[test]
    fn validates_capacity_and_observes_cancellation_first() {
        let registry = ChannelRegistry::new();
        assert!(registry.create(0).is_err());
        let handle = registry.create(1).expect("channel");
        assert!(matches!(
            registry.send(handle, Value::Unit, true, None),
            Ok(SendState::Cancelled)
        ));
        assert!(matches!(
            registry.receive(handle, true, None),
            Ok(ReceiveState::Cancelled)
        ));
    }

    #[test]
    fn close_wakes_a_blocked_sender() {
        let registry = ChannelRegistry::new();
        let handle = registry.create(1).expect("channel");
        assert!(matches!(
            registry.send(handle, Value::Integer(1), false, None),
            Ok(SendState::Sent)
        ));

        std::thread::scope(|scope| {
            let blocked = scope.spawn(|| {
                let mut value = Value::Integer(2);
                loop {
                    match registry
                        .send(handle, value, false, Some(CANCELLATION_POLL_INTERVAL))
                        .expect("send state")
                    {
                        SendState::Pending(pending) => value = pending,
                        outcome => return matches!(outcome, SendState::Closed),
                    }
                }
            });
            std::thread::yield_now();
            assert_eq!(registry.close(handle), Ok(true));
            assert!(blocked.join().expect("sender thread"));
        });
    }

    #[test]
    fn concurrent_senders_and_receivers_deliver_every_value_once() {
        const PRODUCERS: i64 = 4;
        const VALUES_PER_PRODUCER: i64 = 250;

        let registry = Arc::new(ChannelRegistry::new());
        let handle = registry.create(8).expect("channel");
        for value in -8..0 {
            assert!(matches!(
                registry.send(handle, Value::Integer(value), false, None),
                Ok(SendState::Sent)
            ));
        }
        let channel = registry.channel(handle).expect("channel");
        assert_eq!(
            channel
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .values
                .len(),
            8,
            "the bounded queue must reach, but never exceed, its capacity"
        );
        let received = Arc::new(Mutex::new(Vec::new()));

        std::thread::scope(|scope| {
            let consumers = (0..4)
                .map(|_| {
                    let registry = Arc::clone(&registry);
                    let received = Arc::clone(&received);
                    scope.spawn(move || {
                        loop {
                            match registry
                                .receive(handle, false, Some(Duration::from_millis(1)))
                                .expect("receive state")
                            {
                                ReceiveState::Received(Value::Integer(value)) => received
                                    .lock()
                                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                                    .push(value),
                                ReceiveState::Pending => {}
                                ReceiveState::Closed => break,
                                ReceiveState::Received(value) => {
                                    panic!("unexpected channel value: {value:?}")
                                }
                                ReceiveState::Cancelled => {
                                    unreachable!("stress test does not observe cancellation")
                                }
                            }
                        }
                    })
                })
                .collect::<Vec<_>>();
            let producers = (0..PRODUCERS)
                .map(|producer| {
                    let registry = Arc::clone(&registry);
                    scope.spawn(move || {
                        for sequence in 0..VALUES_PER_PRODUCER {
                            let mut value =
                                Value::Integer(producer * VALUES_PER_PRODUCER + sequence);
                            loop {
                                match registry
                                    .send(handle, value, false, Some(Duration::from_millis(1)))
                                    .expect("send state")
                                {
                                    SendState::Sent => break,
                                    SendState::Pending(pending) => value = pending,
                                    SendState::Closed => panic!("channel closed before send"),
                                    SendState::Cancelled => {
                                        unreachable!("stress test does not observe cancellation")
                                    }
                                }
                            }
                        }
                    })
                })
                .collect::<Vec<_>>();

            for producer in producers {
                producer.join().expect("producer");
            }
            assert_eq!(registry.close(handle), Ok(true));
            for consumer in consumers {
                consumer.join().expect("consumer");
            }
        });

        let mut received = Arc::try_unwrap(received)
            .expect("all receiver references were joined")
            .into_inner()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        received.sort_unstable();
        let expected = (-8..PRODUCERS * VALUES_PER_PRODUCER).collect::<Vec<_>>();
        assert_eq!(received, expected);
        let state = channel
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        assert!(state.values.is_empty());
    }

    #[test]
    fn send_notification_wakes_a_waiting_receiver() {
        let registry = ChannelRegistry::new();
        let handle = registry.create(1).expect("channel");
        let channel = registry.channel(handle).expect("channel");
        let waiting = AtomicBool::new(false);

        std::thread::scope(|scope| {
            let waiter = scope.spawn(|| {
                let state = channel
                    .state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                waiting.store(true, Ordering::Release);
                let (_guard, wait) = channel
                    .can_receive
                    .wait_timeout(state, Duration::from_secs(2))
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                wait
            });
            while !waiting.load(Ordering::Acquire) {
                std::thread::yield_now();
            }
            assert!(matches!(
                registry.send(handle, Value::Integer(1), false, None),
                Ok(SendState::Sent)
            ));
            assert!(!waiter.join().expect("receiver waiter").timed_out());
        });
    }

    #[test]
    fn receive_notification_wakes_a_waiting_sender() {
        let registry = ChannelRegistry::new();
        let handle = registry.create(1).expect("channel");
        assert!(matches!(
            registry.send(handle, Value::Integer(1), false, None),
            Ok(SendState::Sent)
        ));
        let channel = registry.channel(handle).expect("channel");
        let waiting = AtomicBool::new(false);

        std::thread::scope(|scope| {
            let waiter = scope.spawn(|| {
                let state = channel
                    .state
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                waiting.store(true, Ordering::Release);
                let (_guard, wait) = channel
                    .can_send
                    .wait_timeout(state, Duration::from_secs(2))
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                wait
            });
            while !waiting.load(Ordering::Acquire) {
                std::thread::yield_now();
            }
            assert!(matches!(
                registry.receive(handle, false, None),
                Ok(ReceiveState::Received(Value::Integer(1)))
            ));
            assert!(!waiter.join().expect("sender waiter").timed_out());
        });
    }
}
