//! Shared diagnostic data model and rendering utilities for the FPAS toolchain.

pub mod codes;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
}

impl SourceLocation {
    #[must_use]
    pub fn new(line: u32, column: u32) -> Self {
        assert!(line > 0, "source line must be 1-based");
        assert!(column > 0, "source column must be 1-based");
        Self { line, column }
    }
}

impl From<(u32, u32)> for SourceLocation {
    fn from((line, column): (u32, u32)) -> Self {
        Self::new(line, column)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub offset: usize,
    pub length: usize,
    pub line: u32,
    pub column: u32,
}

impl SourceSpan {
    #[must_use]
    pub fn new(offset: usize, length: usize, line: u32, column: u32) -> Self {
        assert!(line > 0, "source line must be 1-based");
        assert!(column > 0, "source column must be 1-based");
        Self {
            offset,
            length,
            line,
            column,
        }
    }

    #[must_use]
    pub fn location(self) -> SourceLocation {
        SourceLocation::new(self.line, self.column)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DiagnosticCode(u16);

impl DiagnosticCode {
    pub const MAX_VALUE: u16 = 9999;

    #[must_use]
    pub const fn new(value: u16) -> Self {
        assert!(
            value <= Self::MAX_VALUE,
            "diagnostic code must fit the F0000..F9999 range",
        );
        Self(value)
    }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticStage {
    Lex,
    Parse,
    Sema,
    Compile,
    Runtime,
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticSeverity {
    /// A non-fatal diagnostic that does not block compilation.
    Warning,
    /// A fatal diagnostic that prevents successful compilation or execution.
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub stage: DiagnosticStage,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub help: Option<String>,
    pub span: SourceSpan,
}

impl Diagnostic {
    /// Creates a warning diagnostic.
    #[must_use]
    pub fn warning(
        code: DiagnosticCode,
        stage: DiagnosticStage,
        message: impl Into<String>,
        help: Option<String>,
        span: SourceSpan,
    ) -> Self {
        Self {
            code,
            stage,
            severity: DiagnosticSeverity::Warning,
            message: message.into(),
            help,
            span,
        }
    }

    /// Creates an error diagnostic.
    #[must_use]
    pub fn error(
        code: DiagnosticCode,
        stage: DiagnosticStage,
        message: impl Into<String>,
        help: Option<String>,
        span: SourceSpan,
    ) -> Self {
        Self {
            code,
            stage,
            severity: DiagnosticSeverity::Error,
            message: message.into(),
            help,
            span,
        }
    }
}

#[must_use]
pub fn render(path: &str, diagnostic: &Diagnostic) -> String {
    let severity = match diagnostic.severity {
        DiagnosticSeverity::Warning => "warning",
        DiagnosticSeverity::Error => "error",
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
    use super::{
        Diagnostic, DiagnosticCode, DiagnosticStage, SourceLocation, SourceSpan, render,
    };

    #[test]
    fn source_location_from_tuple() {
        let location = SourceLocation::from((12, 34));
        assert_eq!(location, SourceLocation::new(12, 34));
    }

    #[test]
    fn source_span_location_returns_line_and_column() {
        let span = SourceSpan::new(7, 5, 21, 3);
        assert_eq!(span.location(), SourceLocation::new(21, 3));
    }

    #[test]
    fn render_without_help_line() {
        let diagnostic = Diagnostic::error(
            DiagnosticCode::new(1003),
            DiagnosticStage::Parse,
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
            DiagnosticStage::Parse,
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
    fn render_warning_uses_warning_label() {
        let diagnostic = Diagnostic::warning(
            DiagnosticCode::new(13),
            DiagnosticStage::Lex,
            "Unknown compiler directive `{$R+}`",
            Some("This directive is ignored.".to_string()),
            SourceSpan::new(0, 4, 3, 5),
        );

        let rendered = render("path/to/file.fpas", &diagnostic);
        assert_eq!(
            rendered,
            "path/to/file.fpas:3:5: warning[F0013]: Unknown compiler directive `{$R+}`\n  help: This directive is ignored."
        );
    }

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
