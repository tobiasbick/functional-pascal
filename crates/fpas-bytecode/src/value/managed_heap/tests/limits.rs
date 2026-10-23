//! Pool bounds account for retained bytes and cannot retain cleared value graphs.
use super::*;

fn buffer_bounds<T>() {
    let mut pool = BufferPool::<T>::new();
    for _ in 0..MAX_BUFFERS_PER_BUCKET + 8 {
        pool.recycle(Vec::with_capacity(1));
    }
    assert_eq!(pool.retained_buffers(), MAX_BUFFERS_PER_BUCKET);
    assert_eq!(
        pool.retained_bytes,
        MAX_BUFFERS_PER_BUCKET * mem::size_of::<T>()
    );
    pool.clear();
    let bytes = buffer_bytes::<T>(MAX_BUFFER_CAPACITY);
    for _ in 0..MAX_RETAINED_BYTES / bytes + 8 {
        pool.recycle(Vec::with_capacity(MAX_BUFFER_CAPACITY));
    }
    assert_eq!(pool.retained_bytes, MAX_RETAINED_BYTES);
    assert_eq!(pool.retained_buffers(), MAX_RETAINED_BYTES / bytes);
    let buffer = pool.take(MAX_BUFFER_CAPACITY);
    assert_eq!(pool.retained_bytes, MAX_RETAINED_BYTES - bytes);
    pool.recycle(buffer);
    assert_eq!(pool.retained_bytes, MAX_RETAINED_BYTES);
    pool.clear();
    assert_eq!(pool.retained_bytes, 0);
    assert_eq!(pool.retained_buffers(), 0);
}

#[test]
fn value_buffer_pool_enforces_byte_and_bucket_limits() {
    buffer_bounds::<Value>();
}

#[test]
fn dictionary_buffer_pool_enforces_byte_and_bucket_limits() {
    buffer_bounds::<(Value, Value)>();
}

#[test]
fn string_pool_enforces_combined_bucket_bytes_and_rejects_oversized_buffers() {
    let mut pool = StringPool::new();
    pool.recycle(String::with_capacity(MAX_BUFFER_CAPACITY + 1));
    assert_eq!(pool.retained_bytes, 0);
    for _ in 0..MAX_BUFFERS_PER_BUCKET + 8 {
        pool.recycle(String::with_capacity(1));
    }
    assert_eq!(pool.buckets[0].len(), MAX_BUFFERS_PER_BUCKET);
    assert_eq!(pool.retained_bytes, MAX_BUFFERS_PER_BUCKET);
    pool.clear();
    for _ in 0..MAX_BUFFERS_PER_BUCKET / 2 + 1 {
        pool.recycle(String::with_capacity(MAX_BUFFER_CAPACITY));
    }
    for _ in 0..MAX_BUFFERS_PER_BUCKET + 8 {
        pool.recycle(String::with_capacity(MAX_BUFFER_CAPACITY / 2));
    }
    assert_eq!(pool.retained_bytes, MAX_RETAINED_BYTES);
    let buffer = pool.take(MAX_BUFFER_CAPACITY);
    assert_eq!(
        pool.retained_bytes,
        MAX_RETAINED_BYTES - MAX_BUFFER_CAPACITY
    );
    pool.recycle(buffer);
    pool.recycle(String::with_capacity(MAX_BUFFER_CAPACITY + 1));
    assert_eq!(pool.retained_bytes, MAX_RETAINED_BYTES);
    pool.clear();
    assert_eq!(pool.retained_bytes, 0);
    assert!(pool.buckets.iter().all(Vec::is_empty));
}

#[test]
fn payload_box_pool_is_bounded_and_retains_no_values() {
    PAYLOAD_BOXES.with_borrow_mut(Vec::clear);
    let domain = crate::MemoryDomain::new();
    let cell = domain
        .new_cell(Value::Str("large payload".repeat(8192).into()))
        .unwrap();
    let weak = cell.downgrade();
    let mut nested = Value::Cell(cell);
    for _ in 0..MAX_PAYLOAD_BOXES + 8 {
        nested = Value::option_some(nested);
    }
    crate::release_values(vec![nested]);
    assert!(weak.upgrade().is_none());
    PAYLOAD_BOXES.with_borrow(|boxes| {
        assert_eq!(boxes.len(), MAX_PAYLOAD_BOXES);
        assert!(boxes.iter().all(|value| matches!(**value, Value::Unit)));
    });
    let payload = managed_payload_box(Value::Integer(42));
    assert_eq!(*payload, Value::Integer(42));
    PAYLOAD_BOXES.with_borrow(|boxes| assert_eq!(boxes.len(), MAX_PAYLOAD_BOXES - 1));
    drop(payload);
    PAYLOAD_BOXES.with_borrow_mut(Vec::clear);
}
