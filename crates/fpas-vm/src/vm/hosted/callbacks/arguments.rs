//! Inline storage for the small visible argument lists used by hosted callbacks.

use fpas_bytecode::Value;

/// Callback arguments whose supported arity never exceeds three values.
pub(super) struct CallbackArguments {
    values: [Value; 3],
    len: usize,
}

impl CallbackArguments {
    /// Construct an empty callback argument list.
    pub(super) const fn empty() -> Self {
        Self::new([Value::Unit, Value::Unit, Value::Unit], 0)
    }

    /// Construct a one-value callback argument list.
    pub(super) fn one(value: Value) -> Self {
        Self::new([value, Value::Unit, Value::Unit], 1)
    }

    /// Construct a two-value callback argument list.
    pub(super) fn two(first: Value, second: Value) -> Self {
        Self::new([first, second, Value::Unit], 2)
    }

    /// Construct a three-value callback argument list.
    pub(super) fn three(first: Value, second: Value, third: Value) -> Self {
        Self::new([first, second, third], 3)
    }

    const fn new(values: [Value; 3], len: usize) -> Self {
        Self { values, len }
    }

    /// Borrow the initialized values as a contiguous slice.
    pub(super) fn as_slice(&self) -> &[Value] {
        &self.values[..self.len]
    }

    /// Return the visible callback arity.
    pub(super) fn len(&self) -> usize {
        self.len
    }

    /// Move the initialized values out in argument order.
    pub(super) fn into_values(self) -> impl ExactSizeIterator<Item = Value> {
        self.values.into_iter().take(self.len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_zero_through_three_arguments_without_dynamic_storage() {
        assert!(CallbackArguments::empty().as_slice().is_empty());
        assert_eq!(
            CallbackArguments::one(Value::Integer(1)).as_slice(),
            &[Value::Integer(1)]
        );
        assert_eq!(
            CallbackArguments::two(Value::Integer(1), Value::Integer(2)).as_slice(),
            &[Value::Integer(1), Value::Integer(2)]
        );
        assert_eq!(
            CallbackArguments::three(Value::Integer(1), Value::Integer(2), Value::Integer(3),)
                .as_slice(),
            &[Value::Integer(1), Value::Integer(2), Value::Integer(3)]
        );
    }
}
