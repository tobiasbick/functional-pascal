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
        Self::new((**self).clone())
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
        *value = Value::Unit;
        recycle_payload_box(value);
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
}
