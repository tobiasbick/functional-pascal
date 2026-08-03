//! Stable strongly typed FPAS diagnostic codes.

use core::fmt;

/// Error returned when a numeric value cannot be represented as an FPAS diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidDiagnosticCode {
    value: u16,
}

impl InvalidDiagnosticCode {
    /// Returns the rejected numeric value.
    #[must_use]
    pub const fn value(self) -> u16 {
        self.value
    }
}

impl fmt::Display for InvalidDiagnosticCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "diagnostic code {} is outside the F0000..F9999 range",
            self.value
        )
    }
}

impl std::error::Error for InvalidDiagnosticCode {}

/// A stable FPAS diagnostic code in the `F0000` to `F9999` range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticCode(u16);

impl DiagnosticCode {
    /// Highest numeric value representable as an FPAS diagnostic code.
    pub const MAX_VALUE: u16 = 9999;

    /// Creates a diagnostic code from a statically known numeric value.
    ///
    /// Prefer [`Self::try_new`] for dynamic or untrusted values.
    ///
    /// # Panics
    ///
    /// Panics when `value` is greater than [`Self::MAX_VALUE`].
    #[must_use]
    pub const fn new(value: u16) -> Self {
        assert!(
            value <= Self::MAX_VALUE,
            "diagnostic code must fit the F0000..F9999 range",
        );
        Self(value)
    }

    /// Tries to create a diagnostic code from a dynamic numeric value.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidDiagnosticCode`] when `value` is greater than [`Self::MAX_VALUE`].
    pub const fn try_new(value: u16) -> Result<Self, InvalidDiagnosticCode> {
        if value <= Self::MAX_VALUE {
            Ok(Self(value))
        } else {
            Err(InvalidDiagnosticCode { value })
        }
    }

    /// Returns the numeric value of the diagnostic code.
    #[must_use]
    pub const fn value(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for DiagnosticCode {
    type Error = InvalidDiagnosticCode;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

impl core::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "F{:04}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{DiagnosticCode, InvalidDiagnosticCode};

    #[test]
    fn diagnostic_code_formats_as_fxxxx() {
        assert_eq!(DiagnosticCode::new(1).to_string(), "F0001");
        assert_eq!(DiagnosticCode::new(9999).to_string(), "F9999");
    }

    #[test]
    #[should_panic(expected = "diagnostic code must fit the F0000..F9999 range")]
    fn diagnostic_code_rejects_out_of_range_values() {
        let _ = DiagnosticCode::new(10000);
    }

    #[test]
    fn diagnostic_code_try_new_reports_the_rejected_value() {
        assert_eq!(
            DiagnosticCode::try_new(10000),
            Err(InvalidDiagnosticCode { value: 10000 })
        );
    }
}
