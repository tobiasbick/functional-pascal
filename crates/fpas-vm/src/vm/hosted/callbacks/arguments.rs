//! Inline storage for the small visible argument lists used by hosted callbacks.

use fpas_bytecode::Value;

/// Callback arguments whose supported arity never exceeds three values.
pub(super) enum CallbackArguments {
    Empty,
    One(Value),
    Two([Value; 2]),
    Three([Value; 3]),
}

impl CallbackArguments {
    /// Construct an empty callback argument list.
    pub(super) const fn empty() -> Self {
        Self::Empty
    }

    /// Construct a one-value callback argument list.
    pub(super) fn one(value: Value) -> Self {
        Self::One(value)
    }

    /// Construct a two-value callback argument list.
    pub(super) fn two(first: Value, second: Value) -> Self {
        Self::Two([first, second])
    }

    /// Construct a three-value callback argument list.
    pub(super) fn three(first: Value, second: Value, third: Value) -> Self {
        Self::Three([first, second, third])
    }

    /// Borrow the initialized values as a contiguous slice.
    pub(super) fn as_slice(&self) -> &[Value] {
        match self {
            Self::Empty => &[],
            Self::One(value) => std::slice::from_ref(value),
            Self::Two(values) => values,
            Self::Three(values) => values,
        }
    }

    /// Return the visible callback arity.
    pub(super) fn len(&self) -> usize {
        self.as_slice().len()
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
