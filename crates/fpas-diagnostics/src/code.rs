//! Stable strongly typed FPAS diagnostic codes.

/// A stable FPAS diagnostic code in the `F0000` to `F9999` range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DiagnosticCode(u16);

impl DiagnosticCode {
    /// Highest numeric value representable as an FPAS diagnostic code.
    pub const MAX_VALUE: u16 = 9999;

    /// Creates a diagnostic code from its numeric value.
    #[must_use]
    pub const fn new(value: u16) -> Self {
        assert!(
            value <= Self::MAX_VALUE,
            "diagnostic code must fit the F0000..F9999 range",
        );
        Self(value)
    }

    /// Returns the numeric value of the diagnostic code.
    #[must_use]
    pub const fn value(self) -> u16 {
        self.0
    }
}

impl core::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "F{:04}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::DiagnosticCode;

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
}