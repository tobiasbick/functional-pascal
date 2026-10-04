//! Obsolete construction is rejected without preserving executable fields.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`.

use super::check_errors;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;

#[test]
fn obsolete_records_report_resolved_targets_in_nested_value_contexts() {
    for (declarations, body, target) in [
        ("", "var P: Point := record X := 1; end record;", "Point"),
        (
            "type Alias = Point;",
            "var P: Alias := record X := 1; end record;",
            "Point",
        ),
        (
            "procedure Draw(P: Point); begin null; end procedure;",
            "Draw(record X := 1; end record);",
            "Point",
        ),
        (
            "",
            "var P: option of (Point) := Option.Some(record X := 1; end record);",
            "Point",
        ),
        (
            "",
            "var P: array of (Point) := [record X := 1; end record];",
            "Point",
        ),
        (
            "type Holder = record Child: Point; end record;",
            "var P: Holder := Holder(Child := record X := 1; end record);",
            "Point",
        ),
        (
            "function Origin(): Point; begin return record X := 1; end record; end function;",
            "null;",
            "Point",
        ),
    ] {
        let source = format!(
            "program T; type Point = record X: integer; end record; {declarations} begin {body} end program;"
        );
        let errors = check_errors(&source);
        assert_eq!(errors.len(), 1, "{source}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TYPE_MISMATCH);
        assert_eq!(errors[0].message, "Anonymous record literals are obsolete");
        assert!(
            errors[0]
                .help
                .as_deref()
                .is_some_and(|hint| hint.contains(&format!("`{target}`"))),
            "{errors:#?}"
        );
        let span = errors[0].span.expect("located obsolete construction");
        assert_eq!(
            &source[span.offset()..span.offset() + span.length()],
            "record X := 1; end record"
        );
    }
}

#[test]
fn obsolete_record_without_context_does_not_guess_a_declared_type() {
    let errors = check_errors(
        "program T; type Point = record X: integer; end record; begin discard record X := Unknown(); end record; end program;",
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    let hint = errors[0].help.as_deref().expect("construction hint");
    assert!(hint.contains("TypeName(Field := Value"));
    assert!(hint.contains("No record target can be resolved"));
    assert!(!hint.contains("Point"));
}
