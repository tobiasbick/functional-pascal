//! Uniquely owned wrapper payloads backed by the bounded runtime heap.

use std::fmt;
use std::ops::{Deref, DerefMut};

use super::Value;
use super::managed_heap::{managed_payload_box, recycle_payload_box};

/// A uniquely owned `Result` or `Option` payload.
///
/// Dropped payload boxes return to a bounded thread-local pool. Cloning preserves the previous
/// deep-copy behavior while taking its storage from that pool when possible.
#[derive(Debug)]
pub struct ValuePayload {
    value: Option<Box<Value>>,
}

impl ValuePayload {
    /// Store one runtime value in a reusable payload box.
    #[must_use]
    pub fn new(value: Value) -> Self {
        Self {
            value: Some(managed_payload_box(value)),
        }
    }

    /// Consume the owner and return its payload value.
    #[must_use]
    pub fn into_inner(mut self) -> Value {
        let Some(mut value) = self.value.take() else {
            unreachable!("payload owner must contain a value");
        };
        let inner = std::mem::replace(&mut *value, Value::Unit);
        recycle_payload_box(value);
        inner
    }
}

impl Clone for ValuePayload {
    fn clone(&self) -> Self {
        enum Wrapper {
            Some,
            Ok,
            Error,
        }

        let mut wrappers = Vec::new();
        let mut current = &**self;
        loop {
            let (kind, inner) = match current {
                Value::OptionSome(inner) => (Wrapper::Some, inner),
                Value::ResultOk(inner) => (Wrapper::Ok, inner),
                Value::ResultError(inner) => (Wrapper::Error, inner),
                _ => break,
            };
            wrappers.push(kind);
            current = inner;
        }
        let mut value = current.clone();
        for kind in wrappers.into_iter().rev() {
            value = match kind {
                Wrapper::Some => Value::option_some(value),
                Wrapper::Ok => Value::result_ok(value),
                Wrapper::Error => Value::result_error(value),
            };
        }
        Self::new(value)
    }
}

impl Deref for ValuePayload {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        let Some(value) = self.value.as_deref() else {
            unreachable!("payload owner must contain a value");
        };
        value
    }
}

impl DerefMut for ValuePayload {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Some(value) = self.value.as_deref_mut() else {
            unreachable!("payload owner must contain a value");
        };
        value
    }
}

impl AsRef<Value> for ValuePayload {
    fn as_ref(&self) -> &Value {
        self
    }
}

impl AsMut<Value> for ValuePayload {
    fn as_mut(&mut self) -> &mut Value {
        self
    }
}

impl fmt::Display for ValuePayload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, formatter)
    }
}

impl Drop for ValuePayload {
    fn drop(&mut self) {
        let Some(mut value) = self.value.take() else {
            return;
        };
        let inner = std::mem::replace(&mut *value, Value::Unit);
        recycle_payload_box(value);
        super::drain::release_value(inner);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropped_payload_box_is_reused_and_cleared() {
        let first = ValuePayload::new(Value::Integer(7));
        let first_address = std::ptr::from_ref::<Value>(&first);
        drop(first);

        let second = ValuePayload::new(Value::Boolean(true));
        let second_address = std::ptr::from_ref::<Value>(&second);

        assert_eq!(first_address, second_address);
        assert!(matches!(*second, Value::Boolean(true)));
    }

    #[test]
    fn consuming_payload_returns_value_and_recycles_box() {
        let payload = ValuePayload::new(Value::Integer(11));
        let first_address = std::ptr::from_ref::<Value>(&payload);

        assert_eq!(payload.into_inner(), Value::Integer(11));

        let reused = ValuePayload::new(Value::Unit);
        assert_eq!(first_address, std::ptr::from_ref::<Value>(&reused));
    }

    #[test]
    fn recycling_nested_payloads_does_not_reenter_a_borrowed_pool() {
        let nested = ValuePayload::new(Value::option_some(Value::Integer(3)));

        drop(nested);

        assert!(matches!(*ValuePayload::new(Value::Unit), Value::Unit));
    }

    #[test]
    fn deep_mixed_wrappers_clone_and_release_iteratively_preserving_cell_aliases() {
        std::thread::Builder::new()
            .stack_size(128 * 1024)
            .spawn(|| {
                let domain = crate::MemoryDomain::new();
                let cell = domain.new_cell(Value::Integer(1)).unwrap();
                let weak = cell.downgrade();
                let mut source = Value::Cell(cell.clone());
                for index in 0..30_000 {
                    source = match index % 3 {
                        0 => Value::option_some(source),
                        1 => Value::result_ok(source),
                        _ => Value::result_error(source),
                    };
                }
                let copy = source.clone();
                let mut current = &copy;
                for index in (0..30_000).rev() {
                    current = match (index % 3, current) {
                        (0, Value::OptionSome(inner))
                        | (1, Value::ResultOk(inner))
                        | (2, Value::ResultError(inner)) => inner,
                        _ => panic!("cloning changed wrapper order"),
                    };
                }
                let Value::Cell(copied_cell) = current else {
                    panic!("cloning changed the terminal value");
                };
                assert!(cell.ptr_eq(copied_cell));
                drop(cell);
                crate::release_values(vec![source]);
                assert!(weak.upgrade().is_some());
                crate::release_values(vec![copy]);
                assert!(weak.upgrade().is_none());
                assert_eq!(domain.registered_count(), 0);
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn ordinary_drop_releases_deep_mixed_wrappers_on_a_small_stack() {
        std::thread::Builder::new()
            .stack_size(128 * 1024)
            .spawn(|| {
                let domain = crate::MemoryDomain::new();
                let cell = domain.new_cell(Value::Integer(1)).unwrap();
                let weak = cell.downgrade();
                let mut value = Value::Cell(cell);
                for index in 0..30_000 {
                    value = match index % 3 {
                        0 => Value::option_some(value),
                        1 => Value::result_ok(value),
                        _ => Value::result_error(value),
                    };
                }
                drop(value);
                assert!(weak.upgrade().is_none());
                assert_eq!(domain.registered_count(), 0);
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
