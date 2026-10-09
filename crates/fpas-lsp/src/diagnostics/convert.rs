//! Conversion from compiler diagnostics to LSP diagnostics.

use std::fmt;

use fpas_diagnostics::{
    Diagnostic as FpasDiagnostic, DiagnosticSeverity as FpasDiagnosticSeverity, FileDiagnostic,
};
use fpas_language_service::DocumentSnapshot;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Range};

use crate::convert::{PositionConversionError, byte_offset_to_position};

/// Converts a located record while retaining producer details in diagnostic data.
///
/// **Documentation:** `docs/pascal/tools/diagnostics.md`
pub(crate) fn diagnostic_to_lsp(
    snapshot: &DocumentSnapshot,
    diagnostic: &FpasDiagnostic,
) -> Result<Diagnostic, DiagnosticConversionError> {
    let span = diagnostic
        .span
        .ok_or(DiagnosticConversionError::MissingLocation)?;
    if span.source_id() != 0 {
        return Err(DiagnosticConversionError::ForeignSource {
            source_id: span.source_id(),
        });
    }
    located_diagnostic_to_lsp(snapshot, diagnostic)
}

/// Converts a file-attributed project record with its producer-local source ID intact.
///
/// **Documentation:** `docs/pascal/tools/diagnostics.md`
pub(crate) fn file_diagnostic_to_lsp(
    snapshot: &DocumentSnapshot,
    record: &FileDiagnostic,
) -> Result<Diagnostic, DiagnosticConversionError> {
    if record.path.as_deref() != Some(snapshot.path()) {
        return Err(DiagnosticConversionError::ForeignFile);
    }
    located_diagnostic_to_lsp(snapshot, &record.diagnostic)
}

fn located_diagnostic_to_lsp(
    snapshot: &DocumentSnapshot,
    diagnostic: &FpasDiagnostic,
) -> Result<Diagnostic, DiagnosticConversionError> {
    let span = diagnostic
        .span
        .ok_or(DiagnosticConversionError::MissingLocation)?;
    let start_offset = start_offset(snapshot, diagnostic)?;
    let end_offset = start_offset
        .checked_add(if span.is_synthetic() {
            0
        } else {
            span.length()
        })
        .ok_or(DiagnosticConversionError::InvalidSpan)?;
    let range = Range::new(
        byte_offset_to_position(snapshot, start_offset)?,
        byte_offset_to_position(snapshot, end_offset)?,
    );
    let severity = match diagnostic.severity {
        FpasDiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
        FpasDiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
    };
    let mut message = diagnostic.message.clone();
    if let Some(help) = diagnostic
        .help
        .as_deref()
        .filter(|help| !help.trim().is_empty())
    {
        message.push_str("\n\nHelp: ");
        message.push_str(help);
    }

    Ok(Diagnostic {
        range,
        severity: Some(severity),
        code: Some(NumberOrString::String(diagnostic.code.to_string())),
        source: Some("fpas".to_owned()),
        message,
        data: Some(serde_json::json!({
            "source": snapshot.path().to_string_lossy(),
            "source_id": span.source_id(),
            "message": diagnostic.message,
            "hint": diagnostic.help,
            "expected": diagnostic.expected,
            "found": diagnostic.found,
        })),
        ..Diagnostic::default()
    })
}

fn start_offset(
    snapshot: &DocumentSnapshot,
    diagnostic: &FpasDiagnostic,
) -> Result<usize, DiagnosticConversionError> {
    let span = diagnostic
        .span
        .ok_or(DiagnosticConversionError::MissingLocation)?;
    if !span.is_synthetic() {
        return Ok(span.offset());
    }

    fpas_diagnostics::SourcePosition {
        line: span.line(),
        column: span.column(),
    }
    .offset_in(snapshot.source())
    .ok_or(DiagnosticConversionError::InvalidSpan)
}

#[derive(Debug)]
pub(crate) enum DiagnosticConversionError {
    MissingLocation,
    ForeignSource { source_id: u32 },
    ForeignFile,
    InvalidSpan,
    Position(PositionConversionError),
}

impl fmt::Display for DiagnosticConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLocation => formatter.write_str("diagnostic has no source location"),
            Self::ForeignSource { source_id } => write!(
                formatter,
                "diagnostic belongs to source id {source_id}, not the published document"
            ),
            Self::ForeignFile => formatter.write_str("diagnostic belongs to another source file"),
            Self::InvalidSpan => formatter.write_str("diagnostic span is outside the document"),
            Self::Position(error) => error.fmt(formatter),
        }
    }
}

impl From<PositionConversionError> for DiagnosticConversionError {
    fn from(error: PositionConversionError) -> Self {
        Self::Position(error)
    }
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    reason = "hard-coded conversion fixtures use expect to keep failures local"
)]
mod tests {
    use std::path::Path;

    use fpas_diagnostics::{Diagnostic, DiagnosticCode, FileDiagnostic, SourceSpan};
    use fpas_language_service::DocumentStore;
    use tower_lsp_server::ls_types::{DiagnosticSeverity, NumberOrString, Position, Range};

    use super::{diagnostic_to_lsp, file_diagnostic_to_lsp};

    fn snapshot(source: &str) -> std::sync::Arc<fpas_language_service::DocumentSnapshot> {
        DocumentStore::new()
            .open_document(Path::new("diagnostic-conversion.fpas"), 1, source)
            .expect("test snapshot")
    }

    #[test]
    fn preserves_code_severity_utf16_range_and_help() {
        let snapshot = snapshot("program Demo;\nbegin\n  var Text := '𝄞';\nend.\n");
        let offset = snapshot.source().find("'𝄞'").expect("string offset");
        let diagnostic = Diagnostic::warning(
            DiagnosticCode::new(1005),
            "Character warning",
            Some("Use a supported character.".to_owned()),
            SourceSpan::new(offset, "'𝄞'".len(), 3, 15),
        );

        let converted = diagnostic_to_lsp(&snapshot, &diagnostic).expect("LSP diagnostic");

        assert_eq!(converted.severity, Some(DiagnosticSeverity::WARNING));
        assert_eq!(
            converted.code,
            Some(NumberOrString::String("FP1005".to_owned()))
        );
        assert_eq!(
            converted.range,
            Range::new(Position::new(2, 14), Position::new(2, 18))
        );
        assert_eq!(
            converted.message,
            "Character warning\n\nHelp: Use a supported character."
        );

        let error = Diagnostic::error(
            DiagnosticCode::new(3001),
            "Semantic error",
            None,
            SourceSpan::new(0, 7, 1, 1),
        );
        assert_eq!(
            diagnostic_to_lsp(&snapshot, &error)
                .expect("error diagnostic")
                .severity,
            Some(DiagnosticSeverity::ERROR)
        );
    }

    #[test]
    fn synthetic_scalar_location_converts_to_utf16_without_inventing_an_end() {
        let snapshot = snapshot("é😀x\r\nβ");
        let diagnostic = Diagnostic::error(
            DiagnosticCode::new(5001),
            "runtime error",
            None,
            SourceSpan::synthetic_from_location(fpas_diagnostics::SourceLocation::new(1, 3)),
        );
        let converted = diagnostic_to_lsp(&snapshot, &diagnostic).expect("point location");
        assert_eq!(
            converted.range,
            Range::new(Position::new(0, 3), Position::new(0, 3))
        );
        let missing =
            Diagnostic::error_without_source(DiagnosticCode::new(4101), "missing file", None);
        assert!(matches!(
            diagnostic_to_lsp(&snapshot, &missing),
            Err(super::DiagnosticConversionError::MissingLocation)
        ));
    }

    #[test]
    fn project_record_requires_its_own_file_and_retains_producer_details() {
        let snapshot = snapshot("program Demo; begin end.");
        let mut diagnostic = Diagnostic::error(
            DiagnosticCode::new(2001),
            "Expected semicolon",
            Some("Insert `;`.".into()),
            SourceSpan::new_with_source(0, 7, 1, 1, 37),
        );
        diagnostic.expected = Some(";".into());
        diagnostic.found = Some("begin".into());
        let record = FileDiagnostic::new(diagnostic, Some(snapshot.path().to_path_buf()));
        let converted = file_diagnostic_to_lsp(&snapshot, &record).expect("attributed record");
        let data = converted.data.expect("original producer data");
        assert_eq!(data["source_id"], serde_json::json!(37));
        assert_eq!(data["expected"], serde_json::json!(";"));
        assert_eq!(data["found"], serde_json::json!("begin"));
        assert_eq!(data["hint"], serde_json::json!("Insert `;`."));
        let foreign = FileDiagnostic::new(record.diagnostic, Some("other.fpas".into()));
        assert!(matches!(
            file_diagnostic_to_lsp(&snapshot, &foreign),
            Err(super::DiagnosticConversionError::ForeignFile)
        ));
    }
}
