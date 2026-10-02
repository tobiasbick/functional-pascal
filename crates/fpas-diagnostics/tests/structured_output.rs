//! Shared text/JSON records, Unicode ranges and unavailable source data.

#![allow(
    clippy::unwrap_used,
    reason = "fixed diagnostic fixtures must serialize successfully"
)]

use fpas_diagnostics::{
    Diagnostic, FileDiagnostic, SourceLocation, SourceRange, SourceSpan,
    codes::{
        PARSE_EXPECTED_TOKEN, PROJECT_DUPLICATE_SOURCE_FILE, PROJECT_SOURCE_READ_FAILED,
        RUNTIME_PROGRAM_PANIC,
    },
    render, render_json, render_program_stderr_json, render_without_path,
};
use serde_json::{Value, json};

#[test]
fn missing_source_is_null_without_fabricated_text_coordinates() {
    let diagnostic = Diagnostic::error_without_source(
        PROJECT_SOURCE_READ_FAILED,
        "Cannot read source",
        Some("Check the path.".into()),
    );
    let text = render_without_path(&diagnostic);
    assert_eq!(
        text,
        "error[F5001]: Cannot read source\n  help: Check the path."
    );
    assert_eq!(
        render("missing.fpas", &diagnostic),
        format!("missing.fpas: {text}")
    );
    let encoded = render_json(None, None, &diagnostic).unwrap();
    let decoded: Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        decoded,
        json!({
            "kind": "diagnostic", "code": "F5001", "severity": "error", "phase": "project",
            "source": null, "location": null, "message": "Cannot read source",
            "expected": null, "found": null, "hint": "Check the path."
        })
    );
    assert_eq!(render_json(None, None, &diagnostic).unwrap(), encoded);
}

#[test]
fn unicode_multiline_ranges_have_scalar_columns_and_exclusive_ends() {
    for newline in ["\n", "\r\n", "\r"] {
        let source = format!("é😀x{newline}βz");
        let span = SourceSpan::new_with_source(2, source.len() - 3, 1, 2, 17);
        let diagnostic = Diagnostic::error(PARSE_EXPECTED_TOKEN, "Unexpected range", None, span)
            .with_expected_found(";", "😀x");
        let encoded = render_json(Some("Imported.fpas"), Some(&source), &diagnostic).unwrap();
        let decoded: Value = serde_json::from_str(&encoded).unwrap();
        assert_eq!(
            decoded["location"],
            json!({
                "source_id": 17, "start": {"line": 1, "column": 2},
                "end": {"line": 2, "column": 2}
            })
        );
        assert_eq!(decoded["expected"], ";");
        assert_eq!(decoded["found"], "😀x");
        assert!(
            render("Imported.fpas", &diagnostic).starts_with("Imported.fpas:1:2: error[F1001]")
        );
    }
}

#[test]
fn invalid_boundaries_do_not_panic_or_create_invented_ranges() {
    for span in [SourceSpan::new(1, 1, 1, 1), SourceSpan::new(0, 4, 1, 1)] {
        assert_eq!(SourceRange::resolve(span, Some("é")), None);
    }
    let end = SourceRange::resolve(SourceSpan::new(2, 0, 1, 2), Some("é")).unwrap();
    assert_eq!(end.start, end.end.unwrap());
}

#[test]
fn point_locations_do_not_misinterpret_synthetic_offsets_as_ranges() {
    let diagnostic = Diagnostic::error(
        RUNTIME_PROGRAM_PANIC,
        "panic: failure",
        None,
        SourceSpan::synthetic_from_location(SourceLocation::new_with_source(3, 7, 4)),
    );
    let encoded = render_json(Some("Runtime.fpas"), Some("unrelated prefix"), &diagnostic).unwrap();
    let decoded: Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(
        decoded["location"],
        json!({
            "source_id": 4, "start": {"line": 3, "column": 7}, "end": null
        })
    );
}

#[test]
fn messages_cannot_inject_another_json_line() {
    let diagnostic = Diagnostic::warning(
        PARSE_EXPECTED_TOKEN,
        "first\r\n{\"kind\":\"diagnostic\"}\0",
        Some("quote: \"\\".into()),
        SourceSpan::new(0, 0, 1, 1),
    );
    let encoded = render_json(Some("name\n.fpas"), Some(""), &diagnostic).unwrap();
    assert_eq!(encoded.lines().count(), 1);
    let decoded: Value = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded["message"], diagnostic.message);
    assert_eq!(decoded["hint"], diagnostic.help.unwrap());
    assert_eq!(decoded["severity"], "warning");
}

#[test]
fn scalar_positions_resolve_across_all_line_endings_and_eof() {
    use fpas_diagnostics::SourcePosition;
    for newline in ["\n", "\r\n", "\r"] {
        let source = format!("é😀{newline}β");
        let second_line = "é😀".len() + newline.len();
        assert_eq!(
            SourcePosition { line: 2, column: 1 }.offset_in(&source),
            Some(second_line)
        );
        assert_eq!(
            SourcePosition { line: 2, column: 2 }.offset_in(&source),
            Some(source.len())
        );
        assert_eq!(
            SourcePosition { line: 1, column: 2 }.offset_in(&source),
            Some(2)
        );
        assert_eq!(
            SourcePosition { line: 2, column: 3 }.offset_in(&source),
            None
        );
    }
    assert_eq!(SourcePosition { line: 0, column: 1 }.offset_in(""), None);
}

#[test]
fn file_diagnostics_render_positionless_warnings_with_their_path() {
    let warning = Diagnostic::warning_without_source(
        PROJECT_DUPLICATE_SOURCE_FILE,
        "Duplicate source file was ignored.",
        Some("List each source file once.".into()),
    );
    assert!(warning.is_warning());
    assert_eq!(warning.span, None);

    let located = FileDiagnostic::new(warning.clone(), Some("src/a.fpas".into()));
    assert_eq!(
        located.to_string(),
        "src/a.fpas: warning[F5035]: Duplicate source file was ignored.\n  help: List each source file once."
    );
    assert_eq!(
        FileDiagnostic::new(warning.clone(), None).to_string(),
        render_without_path(&warning)
    );

    let record: Value =
        serde_json::from_str(&render_json(Some("src/a.fpas"), None, &warning).unwrap()).unwrap();
    assert_eq!(record["severity"], "warning");
    assert_eq!(record["phase"], "project");
    assert_eq!(record["location"], Value::Null);
}

#[test]
fn program_output_records_escape_text_and_name_their_kind() {
    let line = render_program_stderr_json("warn \"x\"\t\u{1b}[31m").unwrap();
    assert!(!line.contains('\n'));
    let record: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(
        record,
        json!({"kind": "program-output", "stream": "stderr", "text": "warn \"x\"\t\u{1b}[31m"})
    );
}
