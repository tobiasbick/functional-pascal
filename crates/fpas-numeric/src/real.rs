//! Finite binary64 operations with ordinary rounding and gradual underflow.
//!
//! **Documentation:** `docs/pascal/language/basics/operators.md` and
//! `docs/pascal/std/numeric/math.md`.

use crate::NumericError;

/// Arithmetic operation on two finite binary64 values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    /// Addition.
    Add,
    /// Subtraction.
    Subtract,
    /// Multiplication.
    Multiply,
    /// Division, rejecting both signs of zero.
    Divide,
}

/// Validates a real operand or result without changing its bits, including signed zero.
pub fn finite(value: f64) -> Result<f64, NumericError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(NumericError::NonFiniteReal)
    }
}

/// Applies real arithmetic and rejects nonfinite operands and results.
pub fn binary(operation: Operation, left: f64, right: f64) -> Result<f64, NumericError> {
    finite(left)?;
    finite(right)?;
    finite(match operation {
        Operation::Add => left + right,
        Operation::Subtract => left - right,
        Operation::Multiply => left * right,
        Operation::Divide if right == 0.0 => return Err(NumericError::DivisionByZero),
        Operation::Divide => left / right,
    })
}

/// Converts an integer with binary64 rounding, allowing loss of integer precision.
pub fn from_integer(value: i64) -> f64 {
    value as f64
}

/// Converts a finite real toward zero with an exclusive positive integer bound.
pub fn to_integer(value: f64) -> Result<i64, NumericError> {
    finite(value)?;
    // i64::MAX rounds upward to 2^63 as binary64, so it cannot be an inclusive bound.
    if value < i64::MIN as f64 || value >= -(i64::MIN as f64) {
        return Err(NumericError::IntegerOverflow);
    }
    Ok(value as i64)
}
