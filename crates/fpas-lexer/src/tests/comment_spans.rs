use crate::{collect_comments, lex_with_comments, lex_with_source_id};

#[test]
fn collect_comments_returns_line_comments_in_source_order() {
    let source = "// first\n/// still one slash-slash comment\n// last\nx";
    let comments = collect_comments(source);
    assert_eq!(comments.len(), 3);
    assert_eq!(comments[0].text(source), Some("// first"));
    assert_eq!(
        comments[1].text(source),
        Some("/// still one slash-slash comment")
    );
    assert_eq!(comments[2].text(source), Some("// last"));
}

#[test]
fn is_end_of_line_detects_code_before_comment() {
    let source = "x // trailing\n// alone\n";
    let comments = collect_comments(source);
    assert_eq!(comments[0].is_end_of_line(source), Some(true));
    assert_eq!(comments[1].is_end_of_line(source), Some(false));
}

#[test]
fn is_end_of_line_handles_lf_crlf_and_bare_cr() {
    for newline in ["\n", "\r\n", "\r"] {
        let source = format!("x // trailing{newline}// alone{newline}");
        let comments = collect_comments(&source);
        assert_eq!(comments.len(), 2, "newline {newline:?}");
        assert_eq!(
            comments[0].is_end_of_line(&source),
            Some(true),
            "newline {newline:?}"
        );
        assert_eq!(
            comments[1].is_end_of_line(&source),
            Some(false),
            "newline {newline:?}"
        );
    }
}

#[test]
fn public_comment_helpers_reject_invalid_spans_without_panicking() {
    let source = "é// note";
    let valid = &collect_comments(source)[0];
    assert_eq!(valid.text(source), Some("// note"));
    assert_eq!(valid.end_offset(), Some(source.len()));

    let mut invalid = valid.clone();
    invalid.span.offset = 1;
    invalid.span.length = 1;
    assert_eq!(invalid.text(source), None);
    assert_eq!(invalid.is_end_of_line(source), None);

    invalid.span.offset = usize::MAX;
    invalid.span.length = 1;
    assert_eq!(invalid.end_offset(), None);
    assert_eq!(invalid.text(source), None);
    assert_eq!(invalid.is_end_of_line(source), None);

    assert_eq!(valid.text("short"), None);
    assert_eq!(valid.is_end_of_line("short"), None);
}

#[test]
fn lex_with_comments_matches_collect_comments() {
    let source = "// a\n1";
    let (_, from_lex, _) = lex_with_comments(source);
    assert_eq!(from_lex, collect_comments(source));
}

#[test]
fn lex_with_source_id_tags_comment_spans() {
    let (_, comments, errs) = lex_with_source_id("x // note", 42);
    assert!(errs.is_empty(), "{errs:?}");
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].span.source_id, 42);
}
