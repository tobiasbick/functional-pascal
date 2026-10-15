//! Rendering tests for structured diagnostics.

use fpas_diagnostics::{
    Diagnostic, DiagnosticCode, SourceLocation, SourceSpan, TextStyle, render, render_text,
    render_without_path,
};

fn error(message: &str, help: Option<&str>) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::new(2003),
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
        r"C:\Quellen\größer\datei.fpas:12:8: error[FP2003]: Unerwartetes Zeichen λ"
    );
}

#[test]
fn rendering_escapes_path_controls_without_creating_lines() {
    let rendered = render("unsafe\npath\r\t\u{1b}.fpas", &error("invalid", None));

    assert_eq!(
        rendered,
        "unsafe\\npath\\r\\t\\u{1b}.fpas:12:8: error[FP2003]: invalid"
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
        "main.fpas:12:8: error[FP2003]: first\n  message: second\n  message: third\n  message: fourth\\t\\u{1b}\n  help: try one\n  help: try two\n  help: try three\n  help: "
    );
}

#[test]
fn rendering_without_path_starts_with_the_validated_location() {
    let rendered = render_without_path(&error("invalid", Some("  \r\n\t  ")));

    assert_eq!(rendered, "12:8: error[FP2003]: invalid");
}

#[test]
fn rendering_keeps_an_empty_message_but_omits_empty_help() {
    let rendered = render("main.fpas", &error("", Some("")));

    assert_eq!(rendered, "main.fpas:12:8: error[FP2003]: ");
}

#[test]
fn warning_rendering_uses_the_warning_label() {
    let warning = Diagnostic::warning(
        DiagnosticCode::new(1005),
        "Invalid character code",
        None,
        SourceSpan::new(0, 4, 3, 5),
    );

    assert_eq!(
        render("main.fpas", &warning),
        "main.fpas:3:5: warning[FP1005]: Invalid character code"
    );
}

fn excerpt(source: &str, span: SourceSpan) -> String {
    render_text(
        None,
        Some(source),
        &Diagnostic::error(DiagnosticCode::new(2003), "bad", Some("fix".into()), span),
        TextStyle::Plain,
    )
}

#[test]
fn excerpts_cover_ranges_points_line_endings_and_empty_positions() {
    for source in [
        "first\n  abc\nlast",
        "first\r\n  abc\r\nlast",
        "first\r  abc\rlast",
    ] {
        let offset = source.find("abc").expect("token");
        assert_eq!(
            excerpt(source, SourceSpan::new(offset, 2, 2, 3)),
            "2:3: error[FP2003]: bad\n  2 |   abc\n    |   ^^\n  help: fix"
        );
        assert_eq!(
            excerpt(source, SourceSpan::new(offset, source.len() - offset, 2, 3)),
            "2:3: error[FP2003]: bad\n  2 |   abc\n    |   ^^^ ...\n  help: fix"
        );
        assert_eq!(
            excerpt(
                source,
                SourceSpan::synthetic_from_location(SourceLocation::new(2, 3))
            ),
            "2:3: error[FP2003]: bad\n  2 |   abc\n    |   ^\n  help: fix"
        );
    }
    for (source, span, expected) in [
        (
            "",
            SourceSpan::new(0, 0, 1, 1),
            "1:1: error[FP2003]: bad\n  1 | \n    | ^\n  help: fix",
        ),
        (
            "a\n",
            SourceSpan::new(2, 0, 2, 1),
            "2:1: error[FP2003]: bad\n  2 | \n    | ^\n  help: fix",
        ),
        (
            "abc",
            SourceSpan::new(3, 0, 1, 4),
            "1:4: error[FP2003]: bad\n  1 | abc\n    |    ^\n  help: fix",
        ),
    ] {
        assert_eq!(excerpt(source, span), expected);
    }
}

#[test]
fn excerpts_align_tabs_wide_scalars_combining_marks_and_escaped_controls() {
    for (source, column, spaces) in [
        ("👩‍🔬X", 4, 2),
        ("#\u{fe0f}X", 3, 2),
        ("لاX", 3, 1),
        ("👩‍🔬\tX", 5, 4),
    ] {
        let offset = source.find('X').expect("X");
        assert_eq!(
            excerpt(source, SourceSpan::new(offset, 1, 1, column)),
            format!(
                "1:{column}: error[FP2003]: bad\n  1 | {}X\n    | {}^\n  help: fix",
                if source.contains('\t') {
                    "👩‍🔬  "
                } else {
                    &source[..offset]
                },
                " ".repeat(spaces)
            )
        );
    }
    let source = "\t界e\u{301}\u{1b}X";
    let offset = source.find('X').expect("X");
    assert_eq!(
        excerpt(source, SourceSpan::new(offset, 1, 1, 6)),
        "1:6: error[FP2003]: bad\n  1 |     界e\u{301}\\u{1b}X\n    |              ^\n  help: fix"
    );
    let offset = source.find('\u{301}').expect("combining scalar");
    assert!(excerpt(source, SourceSpan::new(offset, 2, 1, 4)).contains("\n    |        ^\n"));
    assert!(excerpt(source, SourceSpan::new(1, 3, 1, 2)).contains("\n    |     ^^\n"));
}

#[test]
fn excerpts_crop_both_edges_and_clip_ranges_without_splitting_escapes() {
    let sequences = format!("{}X", "#\u{fe0f}".repeat(80));
    let clipped = excerpt(&sequences, SourceSpan::new(sequences.len() - 1, 1, 1, 161));
    let text = clipped
        .lines()
        .nth(1)
        .expect("source")
        .strip_prefix("  1 | ")
        .expect("gutter");
    assert!(
        unicode_width::UnicodeWidthStr::width(text) <= 100,
        "{clipped}"
    );
    let source = format!("{}\u{1b}{}", "界".repeat(80), "z".repeat(150));
    let offset = source.find('\u{1b}').expect("control");
    let rendered = excerpt(
        &source,
        SourceSpan::new(offset, source.len() - offset, 1, 81),
    );
    let lines = rendered.lines().collect::<Vec<_>>();
    let text = lines[1].strip_prefix("  1 | ").expect("source line");
    assert!(text.starts_with("..."));
    assert!(text.ends_with("..."));
    assert!(text.contains("\\u{1b}"));
    assert!(unicode_width::UnicodeWidthStr::width(text) <= 100);
    assert!(lines[2].ends_with(&"^".repeat(46)), "{rendered}");
    let eof = excerpt(&source, SourceSpan::new(source.len(), 0, 1, 232));
    assert!(eof.contains("..."));
    assert_eq!(eof.lines().nth(2).expect("marker").matches('^').count(), 1);
}

#[test]
fn invalid_or_inconsistent_source_ranges_keep_the_original_diagnostic() {
    for span in [
        SourceSpan::new(1, 1, 1, 1),
        SourceSpan::new(0, 99, 1, 1),
        SourceSpan::new(0, 2, 2, 1),
        SourceSpan::synthetic_from_location(SourceLocation::new(9, 1)),
    ] {
        let text = excerpt("ä", span);
        assert!(!text.contains(" | "), "{text}");
        assert!(text.ends_with("\n  help: fix"));
    }
}

#[test]
fn colors_are_scoped_to_severity_and_markers_and_never_trust_source_escapes() {
    for (diagnostic, color) in [
        (error("bad", None), "31"),
        (
            Diagnostic::warning(
                DiagnosticCode::new(1005),
                "bad",
                None,
                SourceSpan::new(0, 1, 1, 1),
            ),
            "33",
        ),
    ] {
        let mut diagnostic = diagnostic;
        diagnostic.span = Some(SourceSpan::new(0, 1, 1, 1));
        let text = render_text(Some("a"), Some("\u{1b}"), &diagnostic, TextStyle::Color);
        let severity = if diagnostic.is_warning() {
            "warning"
        } else {
            "error"
        };
        assert!(text.contains(&format!(
            "\x1b[{color}m{severity}[{}]\x1b[0m: bad",
            diagnostic.code
        )));
        assert!(text.contains(" | \\u{1b}\n"));
        assert!(text.ends_with(&format!("\x1b[{color}m^^^^^^\x1b[0m")));
        assert_eq!(text.matches('\x1b').count(), 4);
    }
}
