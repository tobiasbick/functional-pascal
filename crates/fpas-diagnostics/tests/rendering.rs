//! Rendering tests for structured diagnostics.

use fpas_diagnostics::{Diagnostic, DiagnosticCode, SourceSpan, render, render_without_path};

fn error(message: &str, help: Option<&str>) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::new(1003),
        message,
        help.map(str::to_owned),
        SourceSpan::new(0, 2, 12, 8),
    )
}

#[test]
fn rendering_preserves_printable_unicode_and_windows_paths() {
    let rendered = render(
        r"C:\Quellen\größer\datei.fpas",
        &error("Unerwartetes Zeichen λ", None),
    );

    assert_eq!(
        rendered,
        r"C:\Quellen\größer\datei.fpas:12:8: error[F1003]: Unerwartetes Zeichen λ"
    );
}

#[test]
fn rendering_escapes_path_controls_without_creating_lines() {
    let rendered = render("unsafe\npath\r\t\u{1b}.fpas", &error("invalid", None));

    assert_eq!(
        rendered,
        "unsafe\\npath\\r\\t\\u{1b}.fpas:12:8: error[F1003]: invalid"
    );
    assert_eq!(rendered.lines().count(), 1);
}

#[test]
fn rendering_normalizes_and_prefixes_every_message_and_help_line() {
    let rendered = render(
        "main.fpas",
        &error(
            "first\r\nsecond\rthird\nfourth\t\u{1b}",
            Some("try one\r\ntry two\rtry three\n"),
        ),
    );

    assert_eq!(
        rendered,
        "main.fpas:12:8: error[F1003]: first\n  message: second\n  message: third\n  message: fourth\\t\\u{1b}\n  help: try one\n  help: try two\n  help: try three\n  help: "
    );
}

#[test]
fn rendering_without_path_starts_with_the_validated_location() {
    let rendered = render_without_path(&error("invalid", Some("  \r\n\t  ")));

    assert_eq!(rendered, "12:8: error[F1003]: invalid");
}

#[test]
fn rendering_keeps_an_empty_message_but_omits_empty_help() {
    let rendered = render("main.fpas", &error("", Some("")));

    assert_eq!(rendered, "main.fpas:12:8: error[F1003]: ");
}

#[test]
fn warning_rendering_uses_the_warning_label() {
    let warning = Diagnostic::warning(
        DiagnosticCode::new(5),
        "Invalid character code",
        None,
        SourceSpan::new(0, 4, 3, 5),
    );

    assert_eq!(
        render("main.fpas", &warning),
        "main.fpas:3:5: warning[F0005]: Invalid character code"
    );
}
