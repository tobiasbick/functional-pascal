use std::sync::mpsc;

use super::*;

fn clear_local_pools() {
    VALUE_BUFFERS.with_borrow_mut(BufferPool::clear);
    PAIR_BUFFERS.with_borrow_mut(BufferPool::clear);
    STRING_BUFFERS.with_borrow_mut(StringPool::clear);
}

#[test]
fn recycled_string_buffer_is_cleared_and_reused() {
    clear_local_pools();
    let mut first = managed_string_buffer(8);
    first.push_str("managed");
    let allocation = first.as_ptr();
    recycle_string(&mut first);

    let reused = managed_string_buffer(8);

    assert!(reused.is_empty());
    assert_eq!(reused.as_ptr(), allocation);
}

#[test]
fn recycled_value_buffer_is_cleared_and_reused() {
    clear_local_pools();
    let mut first = managed_value_buffer(4);
    first.push(Value::Integer(1));
    let allocation = first.as_ptr();
    recycle_values(&mut first);

    let reused = managed_value_buffer(4);

    assert!(reused.is_empty());
    assert_eq!(reused.as_ptr(), allocation);
}

#[test]
fn requested_capacity_is_preserved_across_non_power_of_two_buckets() {
    clear_local_pools();
    let mut three_slots = Vec::with_capacity(3);
    three_slots.push(Value::Unit);
    recycle_values(&mut three_slots);

    let buffer = managed_value_buffer(3);

    assert!(buffer.capacity() >= 3);
}

#[test]
fn recycling_nested_values_does_not_reenter_a_borrowed_pool() {
    clear_local_pools();
    let nested = Value::Array(vec![Value::Integer(1)].into());
    let mut outer = vec![nested];

    recycle_values(&mut outer);

    assert!(outer.is_empty());
    assert_eq!(VALUE_BUFFERS.with_borrow(BufferPool::retained_buffers), 2);
}

#[test]
fn oversized_value_buffer_is_not_retained() {
    clear_local_pools();
    let mut oversized = Vec::with_capacity(MAX_BUFFER_CAPACITY + 1);
    oversized.push(Value::Unit);
    recycle_values(&mut oversized);

    let retained = VALUE_BUFFERS.with_borrow(BufferPool::retained_buffers);

    assert_eq!(retained, 0);
}

#[test]
fn recycled_pair_buffer_stays_on_the_recycling_thread() {
    clear_local_pools();
    let (sender, receiver) = mpsc::channel();
    let buffer = managed_pair_buffer(4);
    let allocation = buffer.as_ptr() as usize;
    sender.send(buffer).expect("send pair buffer");

    let reused_allocation = std::thread::spawn(move || {
        clear_local_pools();
        let mut received = receiver.recv().expect("receive pair buffer");
        recycle_pairs(&mut received);
        let reused = managed_pair_buffer(4);
        reused.as_ptr() as usize
    })
    .join()
    .expect("join recycling thread");

    assert_eq!(reused_allocation, allocation);
    assert_eq!(PAIR_BUFFERS.with_borrow(BufferPool::retained_buffers), 0);
}
