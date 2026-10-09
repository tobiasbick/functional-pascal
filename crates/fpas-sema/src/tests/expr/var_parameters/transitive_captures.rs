//! Transitive sibling captures retain reference lifetimes and callable capabilities.
//!
//! Documentation: `docs/pascal/language/functions/var-parameters.md`

use super::{check_errors, check_ok};
use fpas_diagnostics::codes::{
    SEMA_TASK_BOUND_CALLABLE, SEMA_UNSAFE_DISCARD, SEMA_VAR_PARAMETER_ESCAPE,
};

const REFERENCE_HELPERS: &str = r#"
  procedure First();
  begin
    Value := Value + 1;
  end procedure;
  procedure Second();
  begin
    First();
  end procedure;
  procedure Third(Value: string);
  begin
    Second();
  end procedure;
"#;

#[test]
fn sibling_reference_captures_are_deduplicated_and_keep_the_original_declaration() {
    let source = format!(
        "program T; procedure Outer(var Value: integer); {REFERENCE_HELPERS}
begin Third('local'); Second(); end procedure; begin end."
    );
    let (program, errors) = fpas_parser::parse(&source);
    assert!(errors.is_empty(), "{errors:#?}");
    let metadata = crate::analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:#?}", metadata.errors);
    assert_eq!(metadata.nested_routine_captures.len(), 3);
    let captures = metadata
        .nested_routine_captures
        .values()
        .map(|info| {
            assert!(info.task_bound);
            assert_eq!(info.captures.len(), 1);
            let capture = &info.captures[0];
            assert!(capture.reference);
            assert!(capture.mutable);
            assert!(capture.task_free);
            assert_eq!(capture.ty, crate::Ty::Integer);
            capture.declaration
        })
        .collect::<Vec<_>>();
    assert!(captures.iter().all(|span| *span == captures[0]));
}

#[test]
fn sibling_reference_wrappers_cannot_be_returned_assigned_or_passed() {
    for statement in [
        "return Second;",
        "const Escape: procedure() := Second; return Escape;",
        "Accept(Second); return procedure() begin end procedure;",
    ] {
        let source = format!(
            "program T;
procedure Accept(Invoke: procedure()); begin end procedure;
function Make(var Value: integer): procedure(); {REFERENCE_HELPERS}
begin {statement} end function; begin end."
        );
        let errors = check_errors(&source);
        assert_eq!(errors.len(), 1, "{statement}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_VAR_PARAMETER_ESCAPE, "{statement}");
    }
}

#[test]
fn sibling_reference_wrappers_cannot_be_spawned_or_captured_by_anonymous_closures() {
    for statement in [
        "go Second();",
        "const Escape: procedure() := procedure() begin Second(); end procedure;",
        "const Escape: procedure(Value: string) := procedure(Value: string) begin Second(); end procedure;",
    ] {
        let source = format!(
            "program T; uses Std.Tasks;
procedure Outer(var Value: integer); {REFERENCE_HELPERS}
begin {statement} end procedure; begin end."
        );
        let errors = check_errors(&source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_VAR_PARAMETER_ESCAPE),
            "{statement}: {errors:#?}"
        );
    }
}

#[test]
fn local_callable_shadowing_does_not_import_the_siblings_reference_capture() {
    check_ok(
        r#"
program T;
function Make(var Value: integer): procedure();
  procedure First();
  begin
    Value := Value + 1;
  end procedure;
  procedure Second();
  begin
    const First: procedure() := procedure() begin end procedure;
    First();
  end procedure;
begin
  return Second;
end function;
begin
end.
"#,
    );
}

#[test]
fn sibling_wrappers_retain_mutable_and_task_handle_capture_restrictions() {
    for (binding, value_type, statement, expected) in [
        (
            "var Value: integer := 0;",
            "integer",
            "go Second();",
            SEMA_TASK_BOUND_CALLABLE,
        ),
        (
            "const Value: task := go Work();",
            "task",
            "discard Second;",
            SEMA_UNSAFE_DISCARD,
        ),
    ] {
        let source = format!(
            "program T; uses Std.Tasks;
procedure Work(); begin end procedure;
procedure Outer();
  procedure First(); begin const Copy: {value_type} := Value; end procedure;
  procedure Second(); begin First(); end procedure;
begin {binding} {statement} end procedure; begin end."
        );
        let errors = check_errors(&source);
        assert!(
            errors.iter().any(|error| error.code == expected),
            "{errors:#?}"
        );
    }
}
