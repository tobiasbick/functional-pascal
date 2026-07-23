//! Diagnostic records and rendering helpers shared across the toolchain.

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
    pub code: DiagnosticCode,
    pub stage: DiagnosticStage,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub help: Option<String>,
    pub span: SourceSpan,
}

impl DiagnosticCode {
    /// Returns the toolchain stage implied by this code's numeric range.
    #[must_use]
    pub const fn stage(self) -> DiagnosticStage {
        match self.value() {
            1..=12 => DiagnosticStage::Lex,
            1001..=1999 => DiagnosticStage::Parse,
            2001..=2999 => DiagnosticStage::Sema,
            3001..=3999 => DiagnosticStage::Compile,
            4001..=4999 => DiagnosticStage::Runtime,
            _ => DiagnosticStage::Internal,
        }
    }
}

impl Diagnostic {
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
            stage: code.stage(),
            code,
            severity,
            message: message.into(),
            help,
            span,
        }
    }
}

/// Renders a diagnostic as a single-line summary with an optional help line.
#[must_use]
pub fn render(path: &str, diagnostic: &Diagnostic) -> String {
    let severity = if diagnostic.is_warning() {
        "warning"
    } else {
        "error"
    };

    let mut rendered = format!(
        "{path}:{}:{}: {}[{}]: {}",
        diagnostic.span.line, diagnostic.span.column, severity, diagnostic.code, diagnostic.message
    );

    if let Some(help) = diagnostic
        .help
        .as_deref()
        .filter(|help| !help.trim().is_empty())
    {
        rendered.push_str("\n  help: ");
        rendered.push_str(help);
    }

    rendered
}

#[cfg(test)]
mod tests {
    use super::{Diagnostic, DiagnosticStage, render};
    use crate::{DiagnosticCode, SourceSpan};

    #[test]
    fn render_without_help_line() {
        let diagnostic = Diagnostic::error(
            DiagnosticCode::new(1003),
            "Expected `then`, found `do`",
            None,
            SourceSpan::new(0, 2, 12, 8),
        );

        let rendered = render("path/to/file.fpas", &diagnostic);
        assert_eq!(
            rendered,
            "path/to/file.fpas:12:8: error[F1003]: Expected `then`, found `do`"
        );
    }

    #[test]
    fn render_with_help_line() {
        let diagnostic = Diagnostic::error(
            DiagnosticCode::new(1003),
            "Expected `then`, found `do`",
            Some("Insert `then` after the condition.".to_string()),
            SourceSpan::new(0, 2, 12, 8),
        );

        let rendered = render("path/to/file.fpas", &diagnostic);
        assert_eq!(
            rendered,
            "path/to/file.fpas:12:8: error[F1003]: Expected `then`, found `do`\n  help: Insert `then` after the condition."
        );
    }

    #[test]
    fn render_omits_whitespace_only_help() {
        let diagnostic = Diagnostic::error(
            DiagnosticCode::new(1003),
            "Expected `then`, found `do`",
            Some("   \n\t  ".to_string()),
            SourceSpan::new(0, 2, 12, 8),
        );

        let rendered = render("path/to/file.fpas", &diagnostic);
        assert_eq!(
            rendered,
            "path/to/file.fpas:12:8: error[F1003]: Expected `then`, found `do`"
        );
    }

    #[test]
    fn diagnostic_code_stage_matches_numeric_range() {
        assert_eq!(DiagnosticCode::new(5).stage(), DiagnosticStage::Lex);
        assert_eq!(DiagnosticCode::new(1003).stage(), DiagnosticStage::Parse);
        assert_eq!(DiagnosticCode::new(9002).stage(), DiagnosticStage::Internal);
    }

    #[test]
    fn diagnostic_stage_is_derived_from_code() {
        let diagnostic = Diagnostic::error(
            DiagnosticCode::new(3003),
            "arity mismatch",
            None,
            SourceSpan::new(0, 1, 4, 9),
        );
        assert_eq!(diagnostic.stage, DiagnosticStage::Compile);
        assert_eq!(diagnostic.code, DiagnosticCode::new(3003));
    }

    #[test]
    fn render_warning_uses_warning_label() {
        let diagnostic = Diagnostic::warning(
            DiagnosticCode::new(5),
            "Invalid character code in string literal",
            Some("Use decimal digits after `#`, for example `#65` for 'A'.".to_string()),
            SourceSpan::new(0, 4, 3, 5),
        );

        let rendered = render("path/to/file.fpas", &diagnostic);
        assert_eq!(
            rendered,
            "path/to/file.fpas:3:5: warning[F0005]: Invalid character code in string literal\n  help: Use decimal digits after `#`, for example `#65` for 'A'."
        );
    }
}
