//! Nested patterns, payload comparisons, recursive coverage, and unreachable labels.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/exhaustiveness.md`

use super::super::{check_errors, check_ok};
use fpas_diagnostics::DiagnosticCode;
use fpas_diagnostics::codes::{
    SEMA_IMPLICIT_PATTERN_BINDING, SEMA_NON_CONSTANT_EXPRESSION, SEMA_NON_EXHAUSTIVE_CASE,
    SEMA_TYPE_MISMATCH, SEMA_UNREACHABLE_CASE_LABEL,
};

const TYPES: &str = "type Shape = enum Circle(Radius: integer); Rect(Width: integer; Height: integer); Point; end enum;
    type Color = enum Red; Green; end enum;";

fn program(body: &str) -> String {
    format!(
        "program T; {TYPES}
        const Limit: integer := 3;
        function ReadValue(): integer; begin return 1; end function;
        procedure P(R: result of (option of integer, string); O: option of Shape; C: option of Color; B: option of boolean; S: option of string);
        begin
          {body}
        end procedure;
        begin end."
    )
}

fn codes(source: &str, code: DiagnosticCode) -> usize {
    check_errors(source)
        .iter()
        .filter(|error| error.code == code)
        .count()
}

#[test]
fn nested_result_option_and_enum_patterns_are_exhaustive_without_else() {
    check_ok(&program(
        "case R of
           when Ok(Some(const User)): null;
           when Ok(None): null;
           when Error(_): null;
         end case;
         case O of
           when Some(Shape.Circle(const Radius)): null;
           when Some(Shape.Rect(_, 0)): null;
           when Some(Shape.Rect(const W, const H)): null;
           when Some(Shape.Point): null;
           when None: null;
         end case;
         case C of
           when Some(Color.Red): null;
           when Some(Green): null;
           when None: null;
         end case;
         case B of
           when Some(true): null;
           when Some(false): null;
           when None: null;
         end case;",
    ));
}

#[test]
fn missing_nested_patterns_are_named_in_the_diagnostic() {
    let errors = check_errors(&program(
        "case R of
           when Ok(Some(_)): null;
           when Error(_): null;
         end case;",
    ));
    let error = errors
        .iter()
        .find(|error| error.code == SEMA_NON_EXHAUSTIVE_CASE)
        .unwrap_or_else(|| panic!("{errors:#?}"));
    assert!(error.message.contains("Ok(None)"), "{error:#?}");

    let errors = check_errors(&program(
        "case O of
           when Some(Shape.Circle(_)): null;
           when None: null;
         end case;",
    ));
    let error = errors
        .iter()
        .find(|error| error.code == SEMA_NON_EXHAUSTIVE_CASE)
        .unwrap_or_else(|| panic!("{errors:#?}"));
    assert!(
        error.message.contains("Some(Shape.Rect(_, _))"),
        "{error:#?}"
    );
}

#[test]
fn literals_and_guards_do_not_complete_coverage() {
    for body in [
        "case R of when Ok(Some(0)): null; when Ok(None): null; when Error(_): null; end case;",
        "case R of when Ok(Some(const U)) if U > 0: null; when Ok(None): null; when Error(_): null; end case;",
        "case S of when Some('a'): null; when None: null; end case;",
    ] {
        assert_eq!(codes(&program(body), SEMA_NON_EXHAUSTIVE_CASE), 1, "{body}");
    }
    check_ok(&program(
        "case S of when Some('a'): null; when Some(const Other): null; when None: null; end case;",
    ));
}

#[test]
fn covered_labels_are_unreachable() {
    for body in [
        "case R of when Ok(_): null; when Ok(Some(1)): null; when Error(_): null; end case;",
        "case O of when None: null; when None: null; when Some(_): null; end case;",
        "case C of when Some(Red): null; when Some(Green): null; when Some(_): null; when None: null; end case;",
    ] {
        assert_eq!(
            codes(&program(body), SEMA_UNREACHABLE_CASE_LABEL),
            1,
            "{body}"
        );
    }
    check_ok(&program(
        "case O of when Some(const X) if true: null; when Some(_): null; when None: null; end case;",
    ));
}

#[test]
fn payload_comparisons_require_compile_time_constants() {
    check_ok(&program(
        "case R of when Ok(Some(Limit)): null; when Ok(_), Error(_): null; end case;",
    ));
    assert_eq!(
        codes(
            &program(
                "const Computed: integer := ReadValue(); case R of when Ok(Some(Computed)): null; when Ok(_), Error(_): null; end case;"
            ),
            SEMA_NON_CONSTANT_EXPRESSION
        ),
        1
    );
    assert_eq!(
        codes(
            &program(
                "case R of when Ok(Some(Missing)): null; when Ok(_), Error(_): null; end case;"
            ),
            SEMA_IMPLICIT_PATTERN_BINDING
        ),
        1
    );
    assert_eq!(
        codes(
            &program(
                "case R of when Ok(Some('text')): null; when Ok(_), Error(_): null; end case;"
            ),
            SEMA_TYPE_MISMATCH
        ),
        1
    );
}

#[test]
fn explicit_bindings_shadow_constants_and_bare_names_compare() {
    check_ok(&program(
        "case R of
           when Ok(Some(const Limit)): if Limit > 100 then null; end if;
           when Ok(None): null;
           when Error(_): null;
         end case;",
    ));
    check_ok(&program(
        "case O of when Some(Shape.Circle(Limit)): null; when Some(_), None: null; end case;",
    ));
}

#[test]
fn variants_must_match_the_payload_type() {
    assert!(
        codes(
            &program("case R of when Ok(Shape.Point): null; when Ok(_), Error(_): null; end case;"),
            SEMA_TYPE_MISMATCH
        ) >= 1
    );
    assert!(
        codes(
            &program("case O of when Some(Some(_)): null; when Some(_), None: null; end case;"),
            SEMA_TYPE_MISMATCH
        ) >= 1
    );
}

#[test]
fn repeated_payload_comparisons_are_unreachable() {
    for body in [
        "case R of when Ok(Some(1)): null; when Ok(Some(1)): null; when Ok(_), Error(_): null; end case;",
        "case R of when Ok(Some(Limit)): null; when Ok(Some(1 + 2)): null; when Ok(_), Error(_): null; end case;",
        "case S of when Some('ab'): null; when Some('a' + 'b'): null; when Some(_), None: null; end case;",
        "case O of when Some(Shape.Rect(1, 2)): null; when Some(Shape.Rect(1, 2)): null; when Some(_), None: null; end case;",
    ] {
        assert_eq!(
            codes(&program(body), SEMA_UNREACHABLE_CASE_LABEL),
            1,
            "{body}"
        );
    }
    check_ok(&program(
        "case R of when Ok(Some(1)) if true: null; when Ok(Some(1)): null; when Ok(_), Error(_): null; end case;\n         case O of when Some(Shape.Rect(1, 2)): null; when Some(Shape.Rect(1, 3)): null; when Some(Shape.Rect(2, 2)): null; when Some(_), None: null; end case;",
    ));
}

#[test]
fn named_finite_values_complete_nested_coverage() {
    check_ok(&program(
        "const Yes: boolean := true;
         const No: boolean := false;
         const RedChoice: Color := Color.Red;
         const GreenChoice: Color := Color.Green;
         case B of when Some(Yes): null; when Some(No): null; when None: null; end case;
         case C of when Some(RedChoice): null; when Some(GreenChoice): null; when None: null; end case;
         case B of when Some((true)): null; when Some((not true)): null; when None: null; end case;",
    ));
    assert_eq!(
        codes(
            &program(
                "const Yes: boolean := true; case B of when Some(true): null; when Some(Yes): null; when Some(_), None: null; end case;"
            ),
            SEMA_UNREACHABLE_CASE_LABEL
        ),
        1
    );
    assert_eq!(
        codes(
            &program(
                "const RedChoice: Color := Color.Red; case C of when Some(Color.Red): null; when Some(RedChoice): null; when Some(_), None: null; end case;"
            ),
            SEMA_UNREACHABLE_CASE_LABEL
        ),
        1
    );
}

#[test]
fn variant_qualifiers_resolve_to_the_expected_enum() {
    let source = |body: &str| {
        format!(
            "program T;
         type Shape = enum Circle(Radius: integer); Point; end enum;
         type Other = enum Circle(Radius: integer); Point; end enum;
         type ShapeAlias = Shape;
         procedure P(O: option of Shape; S: Shape);
         begin {body} end procedure;
         begin end."
        )
    };
    for body in [
        "case O of when Some(Other.Point): null; when Some(_), None: null; end case;",
        "case O of when Some(Other.Circle(_)): null; when Some(_), None: null; end case;",
        "case S of when Other.Point: null; when Shape.Circle(_), Shape.Point: null; end case;",
    ] {
        assert_eq!(codes(&source(body), SEMA_TYPE_MISMATCH), 1, "{body}");
    }
    assert_eq!(
        codes(
            &source(
                "case O of when Some(Missing.Point): null; when Some(_), None: null; end case;"
            ),
            fpas_diagnostics::codes::SEMA_UNKNOWN_NAME
        ),
        1
    );
    check_ok(&source(
        "case O of when Some(ShapeAlias.Circle(_)): null; when Some(ShapeAlias.Point): null; when None: null; end case;",
    ));
}
