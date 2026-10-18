//! Deterministic single-record JSON diagnostic rendering.

use serde::Serialize;

use crate::{Diagnostic, DiagnosticSeverity, DiagnosticStage, Fix, SourcePosition, SourceRange};

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
    fixes: Vec<FixRecord<'a>>,
}

#[derive(Serialize)]
struct FixRecord<'a> {
    title: &'a str,
    applicability: &'static str,
    edits: Vec<EditRecord<'a>>,
}

#[derive(Serialize)]
struct EditRecord<'a> {
    start: SourcePosition,
    end: SourcePosition,
    new_text: &'a str,
}

/// Resolves every edit against `source`; a fix with an unresolvable edit is left out.
fn fix_record<'a>(fix: &'a Fix, source: Option<&str>) -> Option<FixRecord<'a>> {
    let edits = fix
        .edits
        .iter()
        .map(|edit| {
            let range = SourceRange::resolve(edit.span, source)?;
            Some(EditRecord {
                start: range.start,
                end: range.end?,
                new_text: &edit.new_text,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(FixRecord {
        title: &fix.title,
        applicability: fix.applicability.as_str(),
        edits,
    })
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
        fixes: diagnostic
            .fixes
            .iter()
            .filter_map(|fix| fix_record(fix, source))
            .collect(),
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
