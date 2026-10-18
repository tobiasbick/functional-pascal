//! Fixes attached to semantic habit diagnostics: applying the machine-applicable fixes yields
//! a program that checks without errors, and fixes that change meaning need review.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Fixes)

use super::*;
use fpas_diagnostics::{Applicability, apply_machine_applicable};

/// Applies the machine-applicable fixes of one check and returns the fixed text.
fn fixed(source: &str) -> String {
    let errors = check_errors(source);
    let (text, applied) = apply_machine_applicable(source, errors.iter().flat_map(|e| &e.fixes));
    assert!(applied > 0, "{source}: {errors:#?}");
    text
}

fn program(statements: &str) -> String {
    format!(
        "program P;\n\nuses Std.Console as Console, Std.Conv as Conv;\n\nbegin\n  var Count: integer := 0;\n  const Text := 'abc';\n  const Items := [1, 2];\n  {statements}\n  Console.WriteLn(Conv.IntToStr(Count + Items.Length() + Text.Length()));\nend."
    )
}

#[test]
fn builtin_calls_become_their_fpas_forms() {
    for (statement, expected) in [
        ("Inc(Count);", "Count := Count + 1;"),
        ("Dec(Count, 2);", "Count := Count - 2;"),
        ("Count := Length(Text);", "Count := Text.Length();"),
        (
            "Count := High(Items) * 2;",
            "Count := (Items.Length() - 1) * 2;",
        ),
        ("Count := Low(Items);", "Count := 0;"),
        ("Write(Text);", "Console.WriteText(Text);"),
    ] {
        let source = program(statement);
        let text = fixed(&source);
        assert_eq!(text, program(expected), "{statement}");
        check_ok(&text);
    }
}

#[test]
fn builtins_without_their_unit_or_with_complex_arguments_have_no_fix() {
    for statement in [
        "Count := Length(Items[0].ToString());",
        "discard IntToStr(Count);",
    ] {
        let source = format!(
            "program P;\n\nbegin\n  var Count: integer := 0;\n  const Items := [1, 2];\n  {statement}\nend."
        );
        let errors = check_errors(&source);
        assert!(
            errors.iter().all(|e| e.fixes.is_empty()),
            "{statement}: {errors:#?}"
        );
    }
}

#[test]
fn meaning_changes_need_review() {
    let source = "program P;\n\nbegin\n  const Value: Option of integer := nil;\nend.";
    let errors = check_errors(source);
    let fix = errors.iter().flat_map(|e| &e.fixes).next().expect("fix");
    assert_eq!(fix.applicability, Applicability::NeedsReview);
    assert_eq!(
        fix.apply(source).as_deref(),
        Some(source.replace("nil", "None").as_str())
    );
}

#[test]
fn foreign_types_and_function_name_assignments_are_fixed() {
    let source = "program P;\n\nfunction Half(X: Double): Double;\nbegin\n  Half := X / 2.0;\nend function;\n\nbegin\nend.";
    let mut text = source.to_owned();
    for _ in 0..3 {
        let errors = crate::analyze(&fpas_parser::parse(&text).0);
        let (next, applied) = apply_machine_applicable(&text, errors.iter().flat_map(|e| &e.fixes));
        if applied == 0 {
            break;
        }
        text = next;
    }
    assert_eq!(
        text,
        "program P;\n\nfunction Half(X: real): real;\nbegin\n  return X / 2.0;\nend function;\n\nbegin\nend."
    );
    check_ok(&text);
}

#[test]
fn record_create_calls_become_constructors() {
    let source = "program P;\n\ntype TPoint = record\n  X: integer;\n  Y: integer;\nend record;\n\nbegin\n  const P := TPoint.Create(1, 2);\nend.";
    let text = fixed(source);
    assert_eq!(
        text,
        source.replace("TPoint.Create(1, 2)", "TPoint(X := 1, Y := 2)")
    );
    check_ok(&text);
}
