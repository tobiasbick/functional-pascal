//! Failures shared by static scalar evaluation and constant folding.

/// A checked scalar operation whose result cannot be represented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstantEvaluationError {
    /// An integer operation exceeds the signed 64-bit range.
    IntegerOverflow(&'static str),
    /// Integer division has a zero divisor.
    IntegerDivisionByZero,
    /// Integer remainder has a zero divisor.
    IntegerRemainderByZero,
}

impl std::fmt::Display for ConstantEvaluationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IntegerOverflow(operation) => write!(formatter, "Integer {operation} overflow"),
            Self::IntegerDivisionByZero => formatter.write_str("Integer division by zero"),
            Self::IntegerRemainderByZero => formatter.write_str("Integer modulo by zero"),
        }
    }
}

impl std::error::Error for ConstantEvaluationError {}
