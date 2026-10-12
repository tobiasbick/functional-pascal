//! Checked integer operations used by constant evaluation, folding, and execution.
//!
//! **Documentation:** `docs/pascal/language/basics/operators.md`.

use crate::NumericError;

/// Integer arithmetic operation with a signed 64-bit result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    /// Addition.
    Add,
    /// Subtraction.
    Subtract,
    /// Multiplication.
    Multiply,
    /// Division truncated toward zero.
    Divide,
    /// Remainder with the dividend's sign.
    Remainder,
}

/// Applies an integer operation without wrapping or panicking at domain boundaries.
pub fn binary(operation: Operation, left: i64, right: i64) -> Result<i64, NumericError> {
    let result = match operation {
        Operation::Add => left.checked_add(right),
        Operation::Subtract => left.checked_sub(right),
        Operation::Multiply => left.checked_mul(right),
        Operation::Divide if right == 0 => return Err(NumericError::DivisionByZero),
        Operation::Divide => left.checked_div(right),
        Operation::Remainder if right == 0 => return Err(NumericError::ModuloByZero),
        // Unlike the quotient, this mathematical remainder is representable.
        Operation::Remainder if right == -1 => Some(0),
        Operation::Remainder => left.checked_rem(right),
    };
    result.ok_or(NumericError::IntegerOverflow)
}

/// Negates an integer, rejecting the unrepresentable negation of the minimum value.
pub fn negate(value: i64) -> Result<i64, NumericError> {
    value.checked_neg().ok_or(NumericError::IntegerOverflow)
}
