//! Explicit `const Name` pattern bindings, `_`, and scalar guard bindings.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/syntax.md`

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{
    SEMA_DUPLICATE_DECLARATION, SEMA_IMPLICIT_PATTERN_BINDING, SEMA_INVALID_CASE_BINDING,
    SEMA_NAMED_ARGUMENTS_NOT_SUPPORTED, SEMA_TYPE_MISMATCH, SEMA_UNKNOWN_NAME,
};

const SHAPES: &str =
    "type Shape = enum Circle(Radius: real); Rect(Width: real; Height: real); Point; end enum;";

fn errors_with(source: &str, code: fpas_diagnostics::DiagnosticCode) -> Vec<crate::SemaError> {
    check_errors(source)
        .into_iter()
        .filter(|error| error.code == code)
        .collect()
}

#[test]
fn const_bindings_and_wildcards_are_valid_in_every_pattern_kind() {
    check_ok(&format!(
        "program T; {SHAPES}
        function Area(S: Shape): real;
        begin
          case S of
            when Shape.Circle(const R): return R * R;
            when Shape.Rect(const W, _): return W;
            when Shape.Point: return 0.0;
          end case;
        end function;
        function Describe(R: result of (integer, string); O: option of integer): integer;
        begin
          case R of
            when Ok(const Value): return Value;
            when Error(_): null;
          end case;
          case O of
            when Some(_): return 1;
            when None: return 0;
          end case;
        end function;
        begin end."
    ));
}

#[test]
fn plain_identifier_in_a_payload_reports_the_const_form() {
    for (source, name) in [
        (
            format!("program T; {SHAPES} procedure P(S: Shape); begin case S of when Shape.Circle(R): null; when Shape.Circle(_), Shape.Rect(_, _), Shape.Point: null; end case; end procedure; begin end."),
            "R",
        ),
        (
            "program T; procedure P(O: option of integer); begin case O of when Some(Value): null; when None: null; end case; end procedure; begin end.".to_string(),
            "Value",
        ),
    ] {
        let errors = errors_with(&source, SEMA_IMPLICIT_PATTERN_BINDING);
        assert_eq!(errors.len(), 1, "{errors:#?}");
        assert!(
            errors[0]
                .help
                .as_deref()
                .is_some_and(|hint| hint.contains(&format!("`const {name}`"))),
            "{errors:#?}"
        );
        assert!(
            errors_with(&source, SEMA_UNKNOWN_NAME).is_empty(),
            "recovery binds the name: {source}"
        );
    }
}

#[test]
fn duplicate_bindings_real_comparisons_and_named_fields_are_rejected() {
    let duplicate = format!(
        "program T; {SHAPES} procedure P(S: Shape); begin case S of when Shape.Rect(const A, const A): null; when Shape.Circle(_), Shape.Rect(_, _), Shape.Point: null; end case; end procedure; begin end."
    );
    assert_eq!(errors_with(&duplicate, SEMA_DUPLICATE_DECLARATION).len(), 1);

    let real = format!(
        "program T; {SHAPES} procedure P(S: Shape); begin case S of when Shape.Circle(1.0): null; when Shape.Circle(_), Shape.Rect(_, _), Shape.Point: null; end case; end procedure; begin end."
    );
    assert!(
        errors_with(&real, SEMA_TYPE_MISMATCH)
            .iter()
            .any(|error| error
                .message
                .contains("cannot compare values of type `real`")),
        "{:#?}",
        check_errors(&real)
    );

    let named = format!(
        "program T; {SHAPES} procedure P(S: Shape); begin case S of when Shape.Circle(Radius := const R): null; when Shape.Circle(_), Shape.Rect(_, _), Shape.Point: null; end case; end procedure; begin end."
    );
    assert_eq!(
        errors_with(&named, SEMA_NAMED_ARGUMENTS_NOT_SUPPORTED).len(),
        1
    );
}

#[test]
fn bindings_are_scoped_to_their_arm() {
    let errors = errors_with(
        "program T; procedure P(O: option of integer); begin case O of when Some(const Value): null; when None: null; end case; WriteLn(Value); end procedure; begin end.",
        SEMA_UNKNOWN_NAME,
    );
    assert!(
        errors.iter().any(|error| error.message.contains("Value")),
        "{errors:#?}"
    );
}

#[test]
fn scalar_const_binding_requires_a_guard_and_its_own_arm() {
    check_ok("program T; begin case 3 of when const N if N > 0: null; else null; end case; end.");
    for source in [
        "program T; begin case 3 of when const N: null; end case; end.",
        "program T; begin case 3 of when const N, 2 if N > 0: null; else null; end case; end.",
        "program T; procedure P(O: option of integer); begin case O of when const N if true: null; when Some(_), None: null; end case; end procedure; begin end.",
    ] {
        assert_eq!(
            errors_with(source, SEMA_INVALID_CASE_BINDING).len(),
            1,
            "{source}: {:#?}",
            check_errors(source)
        );
    }
}

#[test]
fn bare_scalar_labels_compare_constants_and_reject_unknown_names() {
    check_ok(
        "program T; const Limit: integer := 3; type Color = enum Red; Green; end enum;
        begin
          case 3 of when Limit if true: null; else null; end case;
          case Color.Red of when Red: null; when Green: null; end case;
        end.",
    );
    let errors = errors_with(
        "program T; begin case 3 of when N if N > 0: null; else null; end case; end.",
        SEMA_IMPLICIT_PATTERN_BINDING,
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert!(
        errors[0]
            .help
            .as_deref()
            .is_some_and(|hint| hint.contains("`when const N if Guard:`")),
        "{errors:#?}"
    );
    assert!(
        errors_with(
            "program T; begin case 3 of when N if N > 0: null; else null; end case; end.",
            SEMA_UNKNOWN_NAME
        )
        .is_empty()
    );
}
