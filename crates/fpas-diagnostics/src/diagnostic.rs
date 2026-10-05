//! Diagnostic records shared across the toolchain.

use crate::{DiagnosticCode, SourceSpan};

/// The compiler or runtime stage that emitted a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticStage {
    /// Lexical analysis.
    Lex,
    /// Syntax parsing.
    Parse,
    /// Semantic analysis.
    Sema,
    /// Bytecode compilation.
    Compile,
    /// Program execution.
    Runtime,
    /// Project loading and build orchestration.
    Project,
    /// An invariant failure inside the toolchain.
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
    pub span: Option<SourceSpan>,
    /// Expected token or value, when the producer can identify it precisely.
    pub expected: Option<Box<str>>,
    /// Actual token or value, when the producer can identify it precisely.
    pub found: Option<Box<str>>,
}

impl DiagnosticCode {
    /// Returns the toolchain stage implied by this code's numeric range.
    #[must_use]
    pub const fn stage(self) -> DiagnosticStage {
        match self.value() {
            1000..=1999 => DiagnosticStage::Lex,
            2000..=2999 => DiagnosticStage::Parse,
            3000..=3999 => DiagnosticStage::Sema,
            4000..=4099 => DiagnosticStage::Compile,
            4100..=4999 => DiagnosticStage::Project,
            5000..=5999 => DiagnosticStage::Runtime,
            _ => DiagnosticStage::Internal,
        }
    }
}

impl Diagnostic {
    /// Creates an error without inventing a source position.
    #[must_use]
    pub fn error_without_source(
        code: DiagnosticCode,
        message: impl Into<String>,
        help: Option<String>,
    ) -> Self {
        Self {
            code,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            help,
            span: None,
            expected: None,
            found: None,
        }
    }

    /// Creates a warning without inventing a source position.
    #[must_use]
    pub fn warning_without_source(
        code: DiagnosticCode,
        message: impl Into<String>,
        help: Option<String>,
    ) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            ..Self::error_without_source(code, message, help)
        }
    }

    /// Attaches producer-supplied expectation details without parsing the message.
    #[must_use]
    pub fn with_expected_found(
        mut self,
        expected: impl Into<String>,
        found: impl Into<String>,
    ) -> Self {
        self.expected = Some(expected.into().into_boxed_str());
        self.found = Some(found.into().into_boxed_str());
        self
    }

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
            span: Some(span),
            expected: None,
            found: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, DiagnosticStage};
    use crate::{DiagnosticCode, SourceSpan};

    #[test]
    fn diagnostic_code_stage_matches_numeric_range() {
        assert_eq!(DiagnosticCode::new(1005).stage(), DiagnosticStage::Lex);
        assert_eq!(DiagnosticCode::new(2003).stage(), DiagnosticStage::Parse);
        assert_eq!(DiagnosticCode::new(9002).stage(), DiagnosticStage::Internal);
    }

    #[test]
    fn diagnostic_stage_is_derived_from_code() {
        let mut diagnostic = Diagnostic::error(
            DiagnosticCode::new(4003),
            "arity mismatch",
            None,
            SourceSpan::new(0, 1, 4, 9),
        );
        assert_eq!(diagnostic.stage(), DiagnosticStage::Compile);
        assert_eq!(diagnostic.code, DiagnosticCode::new(4003));

        diagnostic.code = DiagnosticCode::new(5001);
        assert_eq!(diagnostic.stage(), DiagnosticStage::Runtime);
    }
}
