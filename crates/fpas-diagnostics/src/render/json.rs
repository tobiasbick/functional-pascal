//! Deterministic single-record JSON diagnostic rendering.

use serde::Serialize;

use crate::{Diagnostic, DiagnosticSeverity, DiagnosticStage, SourceRange};

#[derive(Serialize)]
struct Record<'a> {
    kind: &'static str,
    code: String,
    severity: &'static str,
    phase: &'static str,
    source: Option<&'a str>,
    location: Option<SourceRange>,
    message: &'a str,
    expected: Option<&'a str>,
    found: Option<&'a str>,
    hint: Option<&'a str>,
}

/// Serializes one diagnostic as JSON without a trailing newline.
///
/// `source` must be the text belonging to the diagnostic's source ID. Unknown
/// source names or positions remain null. JSON escapes embedded line breaks,
/// so a caller can safely append one newline for a JSON Lines stream.
/// Documentation: `docs/pascal/tools/diagnostics.md`.
///
/// # Errors
/// Returns a serialization error if the JSON serializer cannot encode the record.
pub fn render_json(
    path: Option<&str>,
    source: Option<&str>,
    diagnostic: &Diagnostic,
) -> Result<String, serde_json::Error> {
    let record = Record {
        kind: "diagnostic",
        code: diagnostic.code.to_string(),
        severity: match diagnostic.severity {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
        },
        phase: match diagnostic.stage() {
            DiagnosticStage::Lex => "lex",
            DiagnosticStage::Parse => "parse",
            DiagnosticStage::Sema => "sema",
            DiagnosticStage::Compile => "compile",
            DiagnosticStage::Runtime => "runtime",
            DiagnosticStage::Project => "project",
            DiagnosticStage::Internal => "internal",
        },
        source: path,
        location: diagnostic
            .span
            .and_then(|span| SourceRange::resolve(span, source)),
        message: &diagnostic.message,
        expected: diagnostic.expected.as_deref(),
        found: diagnostic.found.as_deref(),
        hint: diagnostic.help.as_deref(),
    };
    serde_json::to_string(&record)
}

#[derive(Serialize)]
struct ProgramOutput<'a> {
    kind: &'static str,
    stream: &'static str,
    text: &'a str,
}

/// Serializes one standard-error line written by the program, without a trailing newline.
///
/// The record has `"kind":"program-output"`, so a JSON diagnostic stream never
/// presents program text as a compiler or runtime diagnostic.
/// Documentation: `docs/pascal/tools/diagnostics.md`.
///
/// # Errors
/// Returns a serialization error if the JSON serializer cannot encode the record.
pub fn render_program_stderr_json(text: &str) -> Result<String, serde_json::Error> {
    serde_json::to_string(&ProgramOutput {
        kind: "program-output",
        stream: "stderr",
        text,
    })
}
