//! Diagnostic records shared across the toolchain.

use crate::{DiagnosticCode, SourceSpan};

/// The compiler or runtime stage that emitted a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticStage {
    Lex,
    Parse,
    Sema,
    Compile,
    Runtime,
    Internal,
}

/// The severity level of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticSeverity {
    /// A non-fatal diagnostic that does not block compilation.
    Warning,
    /// A fatal diagnostic that prevents successful compilation or execution.
    Error,
}

/// A structured diagnostic emitted by one stage of the FPAS toolchain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// Stable machine-readable diagnostic code.
    pub code: DiagnosticCode,
    /// Error or warning severity.
    pub severity: DiagnosticSeverity,
    /// Human-readable primary message.
    pub message: String,
    /// Optional actionable correction or explanation.
    pub help: Option<String>,
    /// Source range associated with the diagnostic.
    pub span: SourceSpan,
}

impl DiagnosticCode {
    /// Returns the toolchain stage implied by this code's numeric range.
    #[must_use]
    pub const fn stage(self) -> DiagnosticStage {
        match self.value() {
            1..=13 => DiagnosticStage::Lex,
            1001..=1999 => DiagnosticStage::Parse,
            2001..=2999 => DiagnosticStage::Sema,
            3001..=3999 => DiagnosticStage::Compile,
            4001..=4999 => DiagnosticStage::Runtime,
            _ => DiagnosticStage::Internal,
        }
    }
}

impl Diagnostic {
    /// Returns the toolchain stage derived from this diagnostic's current code.
    #[must_use]
    pub const fn stage(&self) -> DiagnosticStage {
        self.code.stage()
    }

    /// Returns `true` when this diagnostic blocks compilation or execution.
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self.severity, DiagnosticSeverity::Error)
    }

    /// Returns `true` when this diagnostic is non-fatal.
    #[must_use]
    pub fn is_warning(&self) -> bool {
        matches!(self.severity, DiagnosticSeverity::Warning)
    }

    /// Creates a warning diagnostic.
    #[must_use]
    pub fn warning(
        code: DiagnosticCode,
        message: impl Into<String>,
        help: Option<String>,
        span: SourceSpan,
    ) -> Self {
        Self::new(DiagnosticSeverity::Warning, code, message, help, span)
    }

    /// Creates an error diagnostic.
    #[must_use]
    pub fn error(
        code: DiagnosticCode,
        message: impl Into<String>,
        help: Option<String>,
        span: SourceSpan,
    ) -> Self {
        Self::new(DiagnosticSeverity::Error, code, message, help, span)
    }

    fn new(
        severity: DiagnosticSeverity,
        code: DiagnosticCode,
        message: impl Into<String>,
        help: Option<String>,
        span: SourceSpan,
    ) -> Self {
        Self {
            code,
            severity,
            message: message.into(),
            help,
            span,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, DiagnosticStage};
    use crate::{DiagnosticCode, SourceSpan};

    #[test]
    fn diagnostic_code_stage_matches_numeric_range() {
        assert_eq!(DiagnosticCode::new(5).stage(), DiagnosticStage::Lex);
        assert_eq!(DiagnosticCode::new(1003).stage(), DiagnosticStage::Parse);
        assert_eq!(DiagnosticCode::new(9002).stage(), DiagnosticStage::Internal);
    }

    #[test]
    fn diagnostic_code_stage_respects_every_range_boundary() {
        for (code, stage) in [
            (13, DiagnosticStage::Lex),
            (14, DiagnosticStage::Internal),
            (1001, DiagnosticStage::Parse),
            (1999, DiagnosticStage::Parse),
            (2000, DiagnosticStage::Internal),
            (2001, DiagnosticStage::Sema),
            (2999, DiagnosticStage::Sema),
            (3000, DiagnosticStage::Internal),
            (3001, DiagnosticStage::Compile),
            (3999, DiagnosticStage::Compile),
            (4000, DiagnosticStage::Internal),
            (4001, DiagnosticStage::Runtime),
            (4999, DiagnosticStage::Runtime),
            (5000, DiagnosticStage::Internal),
        ] {
            assert_eq!(DiagnosticCode::new(code).stage(), stage);
        }
    }

    #[test]
    fn diagnostic_stage_is_derived_from_code() {
        let mut diagnostic = Diagnostic::error(
            DiagnosticCode::new(3003),
            "arity mismatch",
            None,
            SourceSpan::new(0, 1, 4, 9),
        );
        assert_eq!(diagnostic.stage(), DiagnosticStage::Compile);
        assert_eq!(diagnostic.code, DiagnosticCode::new(3003));

        diagnostic.code = DiagnosticCode::new(4001);
        assert_eq!(diagnostic.stage(), DiagnosticStage::Runtime);
    }
}
