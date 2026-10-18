//! Fixes attached to foreign-token and comment diagnostics: applying the machine-applicable
//! fix yields source that lexes without errors.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Fixes)

use crate::lex;
use fpas_diagnostics::Applicability;

/// Lexes `source`, applies the single diagnostic's fix and returns the fixed text.
fn fixed(source: &str, applicability: Applicability) -> String {
    let (_, errors) = lex(source);
    assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
    let [fix] = errors[0].fixes.as_slice() else {
        panic!("{source}: expected one fix, got {:#?}", errors[0].fixes);
    };
    assert_eq!(fix.applicability, applicability, "{source}");
    fix.apply(source).expect("fix applies to its source")
}

fn assert_machine_fix(source: &str, expected: &str) {
    let text = fixed(source, Applicability::MachineApplicable);
    assert_eq!(text, expected, "{source}");
    let (_, errors) = lex(&text);
    assert!(errors.is_empty(), "{text}: {errors:#?}");
}

fn assert_no_fix(source: &str) {
    let (_, errors) = lex(source);
    assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
    assert!(
        errors[0].fixes.is_empty(),
        "{source}: {:#?}",
        errors[0].fixes
    );
}

#[test]
fn operators_get_spaced_word_replacements() {
    for (source, expected) in [
        ("A && B", "A and B"),
        ("A&&B", "A and B"),
        ("A || B", "A or B"),
        ("A != B", "A <> B"),
        ("!Done", "not Done"),
        ("A == B", "A = B"),
        ("A /= B", "A <> B"),
        ("F(X => 1)", "F(X := 1)"),
        ("'a' & 'b'", "'a' + 'b'"),
    ] {
        assert_machine_fix(source, expected);
    }
}

#[test]
fn pointer_operators_have_no_fix() {
    assert_no_fix("@F");
    assert_no_fix("P^");
}

#[test]
fn double_quoted_strings_become_single_quoted() {
    assert_machine_fix(r#"S := "it's";"#, "S := 'it''s';");
    assert_eq!(
        fixed(r#"S := "a\nb";"#, Applicability::NeedsReview),
        r"S := 'a\nb';"
    );
    assert_no_fix("S := \"open\n");
}

#[test]
fn comments_become_line_comments() {
    assert_machine_fix("X := 1; { note }", "X := 1; // note");
    assert_machine_fix("(* note *)\nX := 1;", "// note\nX := 1;");
    assert_machine_fix(
        "  {\n    first\n    second\n  }\n",
        "  // first\n  // second\n",
    );
    assert_machine_fix("  (* a\r\n  b *)\r\n", "  // a\r\n  // b\r\n");
    assert_machine_fix("-- note\nX := 1;", "// note\nX := 1;");
    assert_machine_fix("{}", "//");
}

#[test]
fn comments_followed_by_code_or_unclosed_have_no_fix() {
    assert_no_fix("{ note } X := 1;");
    assert_no_fix("X := 1; { a\nb }");
    assert_no_fix("{ open");
}

#[test]
fn attribute_length_is_exact_and_bounds_need_review() {
    assert_machine_fix("N := Items'Length;", "N := Items.Length();");
    assert_eq!(
        fixed("N := Items'Last;", Applicability::NeedsReview),
        "N := Items.Length() - 1;"
    );
    assert_no_fix("S := Integer'Image(N);");
}
