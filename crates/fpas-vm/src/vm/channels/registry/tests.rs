use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::{CANCELLATION_POLL_INTERVAL, ChannelRegistry, ReceiveState, SendState};
use crate::vm::shared::wakeups::WakeSignal;

#[test]
fn timed_pending_operations_remove_every_registration() {
    let registry = ChannelRegistry::new();
    let handle = registry.create(1).expect("channel");
    let channel = registry.channel(handle).expect("channel state");
    for _ in 0..1000 {
        assert!(matches!(
            registry.receive(handle, false, Some(Duration::ZERO)),
            Ok(ReceiveState::Pending)
        ));
        assert_eq!(channel.can_receive.registration_count(), 0);
    }
    assert!(matches!(
        registry.send(handle, Value::Integer(7), false, None),
        Ok(SendState::Sent)
    ));
    for _ in 0..1000 {
        assert!(matches!(
            registry.send(handle, Value::Integer(8), false, Some(Duration::ZERO)),
            Ok(SendState::Pending(Value::Integer(8)))
        ));
        assert_eq!(channel.can_send.registration_count(), 0);
    }
}

#[test]
fn two_channel_sources_wake_without_consuming_either_value() {
    let registry = ChannelRegistry::new();
    let first = registry.create(1).expect("first channel");
    let second = registry.create(1).expect("second channel");
    let signal = WakeSignal::new();
    let first_channel = registry.channel(first).unwrap();
    let second_channel = registry.channel(second).unwrap();
    let registrations = [
        first_channel.can_receive.subscribe(&signal),
        second_channel.can_receive.subscribe(&signal),
    ];
    assert!(matches!(
        registry.send(second, Value::Integer(2), false, None),
        Ok(SendState::Sent)
    ));
    assert!(signal.wait(Duration::ZERO));
    assert!(matches!(
        registry.send(first, Value::Integer(1), false, None),
        Ok(SendState::Sent)
    ));
    drop(registrations);
    assert_eq!(first_channel.can_receive.registration_count(), 0);
    assert_eq!(second_channel.can_receive.registration_count(), 0);
    assert!(matches!(
        registry.receive(first, false, None),
        Ok(ReceiveState::Received(Value::Integer(1)))
    ));
    assert!(matches!(
        registry.receive(second, false, None),
        Ok(ReceiveState::Received(Value::Integer(2)))
    ));
}

#[test]
fn close_and_shutdown_notify_both_operation_directions() {
    for shutdown in [false, true] {
        let registry = ChannelRegistry::new();
        let handle = registry.create(1).expect("channel");
        let channel = registry.channel(handle).unwrap();
        let sending = WakeSignal::new();
        let receiving = WakeSignal::new();
        let registrations = [
            channel.can_send.subscribe(&sending),
            channel.can_receive.subscribe(&receiving),
        ];
        if shutdown {
            registry.shutdown();
        } else {
            registry.close(handle).unwrap();
        }
        assert!(sending.wait(Duration::ZERO));
        assert!(receiving.wait(Duration::ZERO));
        drop(registrations);
        assert_eq!(channel.can_send.registration_count(), 0);
        assert_eq!(channel.can_receive.registration_count(), 0);
    }
}
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
                        let mut value = Value::Integer(producer * VALUES_PER_PRODUCER + sequence);
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
            let signal = WakeSignal::new();
            let _registration = channel.can_receive.subscribe(&signal);
            waiting.store(true, Ordering::Release);
            drop(state);
            signal.wait(Duration::from_secs(2))
        });
        while !waiting.load(Ordering::Acquire) {
            std::thread::yield_now();
        }
        assert!(matches!(
            registry.send(handle, Value::Integer(1), false, None),
            Ok(SendState::Sent)
        ));
        assert!(waiter.join().expect("receiver waiter"));
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
            let signal = WakeSignal::new();
            let _registration = channel.can_send.subscribe(&signal);
            waiting.store(true, Ordering::Release);
            drop(state);
            signal.wait(Duration::from_secs(2))
        });
        while !waiting.load(Ordering::Acquire) {
            std::thread::yield_now();
        }
        assert!(matches!(
            registry.receive(handle, false, None),
            Ok(ReceiveState::Received(Value::Integer(1)))
        ));
        assert!(waiter.join().expect("sender waiter"));
    });
}
