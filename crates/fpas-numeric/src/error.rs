/// Failure of a checked numeric operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericError {
    /// The mathematical integer result is outside the signed 64-bit range.
    IntegerOverflow,
    /// The divisor of `/` or `div` is zero.
    DivisionByZero,
    /// The divisor of `mod` is zero.
    ModuloByZero,
    /// A real operand or result is NaN or infinite.
    NonFiniteReal,
}

impl NumericError {
    /// Returns the user-facing description shared by numeric consumers.
    pub const fn message(self) -> &'static str {
        match self {
            Self::IntegerOverflow => "Integer arithmetic overflow",
            Self::DivisionByZero => "Division by zero",
            Self::ModuloByZero => "Modulo by zero",
            Self::NonFiniteReal => "Real operand or result is not finite",
        }
    }

    /// Returns an actionable hint for the failed operation.
    pub const fn hint(self) -> &'static str {
        match self {
            Self::IntegerOverflow => {
                "Keep integer results within -9223372036854775808..9223372036854775807; ordinary arithmetic does not wrap."
            }
            Self::DivisionByZero => {
                "Check the right-hand side before using `div` or `/`; both signs of real zero are invalid divisors."
            }
            Self::ModuloByZero => "Check the right-hand side before using `mod`.",
            Self::NonFiniteReal => {
                "Use finite real operands and keep the result within binary64 range; NaN and infinity are not FPAS real values."
            }
        }
    }
}

impl std::fmt::Display for NumericError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for NumericError {}
