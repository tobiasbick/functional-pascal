//! Bounded thread-local storage for reusable aggregate buffers.

use std::cell::RefCell;
use std::mem;

use super::Value;

const MAX_BUFFER_CAPACITY: usize = 4096;
const MAX_RETAINED_BYTES: usize = 4 * 1024 * 1024;
const MAX_BUFFERS_PER_BUCKET: usize = 1024;
const MAX_PAYLOAD_BOXES: usize = 4096;
const BUCKET_COUNT: usize = 13;

struct BufferPool<T> {
    buckets: [Vec<Vec<T>>; BUCKET_COUNT],
    retained_bytes: usize,
}

struct StringPool {
    buckets: [Vec<String>; BUCKET_COUNT],
    retained_bytes: usize,
}

impl StringPool {
    fn new() -> Self {
        Self {
            buckets: std::array::from_fn(|_| Vec::new()),
            retained_bytes: 0,
        }
    }

    fn take(&mut self, minimum_capacity: usize) -> String {
        let Some(first_bucket) = request_bucket(minimum_capacity) else {
            return String::with_capacity(minimum_capacity);
        };
        for bucket in &mut self.buckets[first_bucket..] {
            let Some(buffer) = bucket.pop() else {
                continue;
            };
            debug_assert!(buffer.capacity() >= minimum_capacity);
            self.retained_bytes = self.retained_bytes.saturating_sub(buffer.capacity());
            return buffer;
        }
        String::with_capacity(minimum_capacity.next_power_of_two().max(1))
    }

    fn recycle(&mut self, buffer: String) {
        debug_assert!(buffer.is_empty());
        let capacity = buffer.capacity();
        let Some(bucket_index) = capacity_bucket(capacity) else {
            return;
        };
        if capacity == 0 || self.buckets[bucket_index].len() >= MAX_BUFFERS_PER_BUCKET {
            return;
        }
        if self.retained_bytes.saturating_add(capacity) > MAX_RETAINED_BYTES {
            return;
        }
        self.retained_bytes += capacity;
        self.buckets[bucket_index].push(buffer);
    }

    #[cfg(test)]
    fn clear(&mut self) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
        self.retained_bytes = 0;
    }
}

impl<T> BufferPool<T> {
    fn new() -> Self {
        Self {
            buckets: std::array::from_fn(|_| Vec::new()),
            retained_bytes: 0,
        }
    }

    fn take(&mut self, minimum_capacity: usize) -> Vec<T> {
        let Some(first_bucket) = request_bucket(minimum_capacity) else {
            return Vec::with_capacity(minimum_capacity);
        };
        for bucket in &mut self.buckets[first_bucket..] {
            let Some(buffer) = bucket.pop() else {
                continue;
            };
            debug_assert!(buffer.capacity() >= minimum_capacity);
            self.retained_bytes = self
                .retained_bytes
                .saturating_sub(buffer_bytes::<T>(buffer.capacity()));
            return buffer;
        }
        Vec::with_capacity(minimum_capacity.next_power_of_two().max(1))
    }

    fn recycle(&mut self, buffer: Vec<T>) {
        debug_assert!(buffer.is_empty());
        let capacity = buffer.capacity();
        let Some(bucket_index) = capacity_bucket(capacity) else {
            return;
        };
        if capacity == 0 || self.buckets[bucket_index].len() >= MAX_BUFFERS_PER_BUCKET {
            return;
        }
        let bytes = buffer_bytes::<T>(capacity);
        if self.retained_bytes.saturating_add(bytes) > MAX_RETAINED_BYTES {
            return;
        }
        self.retained_bytes += bytes;
        self.buckets[bucket_index].push(buffer);
    }

    #[cfg(test)]
    fn clear(&mut self) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
        self.retained_bytes = 0;
    }

    #[cfg(test)]
    fn retained_buffers(&self) -> usize {
        self.buckets.iter().map(Vec::len).sum()
    }
}

fn request_bucket(capacity: usize) -> Option<usize> {
    if capacity > MAX_BUFFER_CAPACITY {
        return None;
    }
    Some(if capacity <= 1 {
        0
    } else {
        usize::BITS as usize - (capacity - 1).leading_zeros() as usize
    })
}

fn capacity_bucket(capacity: usize) -> Option<usize> {
    if capacity > MAX_BUFFER_CAPACITY {
        return None;
    }
    Some(if capacity <= 1 {
        0
    } else {
        usize::BITS as usize - 1 - capacity.leading_zeros() as usize
    })
}

fn buffer_bytes<T>(capacity: usize) -> usize {
    capacity.saturating_mul(mem::size_of::<T>())
}

thread_local! {
    static VALUE_BUFFERS: RefCell<BufferPool<Value>> = RefCell::new(BufferPool::new());
    static PAIR_BUFFERS: RefCell<BufferPool<(Value, Value)>> = RefCell::new(BufferPool::new());
    static STRING_BUFFERS: RefCell<StringPool> = RefCell::new(StringPool::new());
    #[allow(
        clippy::vec_box,
        reason = "the pool intentionally retains individual allocations for payload reuse"
    )]
    static PAYLOAD_BOXES: RefCell<Vec<Box<Value>>> = const { RefCell::new(Vec::new()) };
}

pub(super) fn managed_payload_box(value: Value) -> Box<Value> {
    let mut payload = PAYLOAD_BOXES
        .try_with(|pool| pool.borrow_mut().pop())
        .ok()
        .flatten()
        .unwrap_or_else(|| Box::new(Value::Unit));
    *payload = value;
    payload
}

pub(super) fn recycle_payload_box(payload: Box<Value>) {
    debug_assert!(matches!(*payload, Value::Unit));
    let _ = PAYLOAD_BOXES.try_with(move |pool| {
        let mut pool = pool.borrow_mut();
        if pool.len() < MAX_PAYLOAD_BOXES {
            pool.push(payload);
        }
    });
}

/// Take a cleared value buffer from the current thread's bounded runtime heap.
///
/// Aggregate bodies return eligible buffers when their final shared owner is dropped. Callers
/// should move the returned buffer into a shared aggregate so it can be recycled again.
#[must_use]
pub fn managed_value_buffer(minimum_capacity: usize) -> Vec<Value> {
    VALUE_BUFFERS
        .try_with(|pool| pool.borrow_mut().take(minimum_capacity))
        .unwrap_or_else(|_| Vec::with_capacity(minimum_capacity))
}

pub(super) fn managed_pair_buffer(minimum_capacity: usize) -> Vec<(Value, Value)> {
    PAIR_BUFFERS
        .try_with(|pool| pool.borrow_mut().take(minimum_capacity))
        .unwrap_or_else(|_| Vec::with_capacity(minimum_capacity))
}

pub(super) fn managed_string_buffer(minimum_capacity: usize) -> String {
    STRING_BUFFERS
        .try_with(|pool| pool.borrow_mut().take(minimum_capacity))
        .unwrap_or_else(|_| String::with_capacity(minimum_capacity))
}

pub(super) fn clone_values(values: &[Value]) -> Vec<Value> {
    let mut clone = managed_value_buffer(values.len());
    clone.extend_from_slice(values);
    clone
}

pub(super) fn clone_pairs(values: &[(Value, Value)]) -> Vec<(Value, Value)> {
    let mut clone = managed_pair_buffer(values.len());
    clone.extend_from_slice(values);
    clone
}

pub(super) fn recycle_values(values: &mut Vec<Value>) {
    let mut values = mem::take(values);
    values.clear();
    let _ = VALUE_BUFFERS.try_with(move |pool| pool.borrow_mut().recycle(values));
}

pub(super) fn recycle_pairs(values: &mut Vec<(Value, Value)>) {
    let mut values = mem::take(values);
    values.clear();
    let _ = PAIR_BUFFERS.try_with(move |pool| pool.borrow_mut().recycle(values));
}

pub(super) fn recycle_string(value: &mut String) {
    let mut value = mem::take(value);
    value.clear();
    let _ = STRING_BUFFERS.try_with(move |pool| pool.borrow_mut().recycle(value));
}

#[cfg(test)]
mod tests;
