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
fn recursive_capture_completion_preserves_task_and_discard_boundaries() {
    for statement in [
        "go Work();",
        "const Items: array of procedure() := [Work]; go Items[0]();",
        "const Wrapped: Option of procedure() := Some(Work); if Wrapped is Some(const Extracted) then go Extracted(); end if;",
        "const Wrapped: Option of procedure() := Some(Work); const Extracted: procedure() := Wrapped.Unwrap(); go Extracted();",
        "const Wrapped: procedure() := procedure() begin Work(); end procedure; go Wrapped();",
        "go Alias();",
        "const Queue: channel of procedure() := CreateChannel(1); discard Send(Queue, Work);",
    ] {
        let errors = check_errors(&format!(
            "program T; uses Std.Tasks;
procedure Outer();
  procedure First();
    procedure Alias(); begin Work(); end procedure;
  begin
    const Work: procedure() := procedure() begin First(); end procedure;
    {statement}
    Value := Value + 1;
  end procedure;
begin var Value: integer := 0; end procedure;
begin end."
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TASK_BOUND_CALLABLE),
            "{statement}: {errors:#?}"
        );
    }
    for statement in [
        "discard Work;",
        "const Wrapped: Option of procedure() := Some(Work); discard Wrapped.Unwrap();",
    ] {
        let errors = check_errors(&format!(
            "program T; uses Std.Tasks;
procedure Outer(Value: task);
  procedure First();
  begin
    const Work: procedure() := procedure() begin First(); end procedure;
    {statement}
    const Snapshot: task := Value;
  end procedure;
begin end procedure;
begin end."
        ));
        assert!(
            errors.iter().any(|error| error.code == SEMA_UNSAFE_DISCARD),
            "{errors:#?}"
        );
    }
    check_ok(
        "program T; uses Std.Tasks;
procedure Outer(Value: task);
  procedure First();
  begin
    const Work: procedure() := procedure() begin First(); end procedure;
    discard Some(Work).IsSome();
    const Present: boolean := Some(Work).IsSome();
    discard Present;
    const Snapshot: task := Value;
  end procedure;
begin end procedure;
begin end.",
    );
}

#[test]
fn recursive_routine_values_and_tasks_cannot_escape_reference_captures() {
    for statement in [
        "const Escape: procedure() := First;",
        "go First();",
        "const Escape: procedure() := procedure() begin const Copy: procedure() := First; end procedure;",
        "const Escape: procedure() := procedure() begin First(); end procedure;",
    ] {
        let source = format!(
            "program T; uses Std.Tasks;
procedure Outer(var Value: integer);
  procedure First();
  begin
    {statement}
    Value := Value + 1;
  end procedure;
begin First(); end procedure;
begin end."
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
fn recursive_reference_dependencies_reach_anonymous_and_named_wrappers() {
    for (bridge_body, statement) in [
        (
            "First();",
            "const Escape: procedure() := procedure() begin Bridge(); end procedure;",
        ),
        ("First();", "const Escape: procedure() := Bridge;"),
        ("First();", "go Bridge();"),
        ("const Copy: procedure() := Bridge; First();", "null;"),
        (
            "const Copy: procedure() := procedure() begin First(); end procedure;",
            "null;",
        ),
    ] {
        let source = format!(
            "program T; uses Std.Tasks;
procedure Outer(var Value: integer);
  procedure First();
    procedure Bridge();
    begin {bridge_body} end procedure;
  begin
    {statement}
    Value := Value + 1;
  end procedure;
begin First(); end procedure;
begin end."
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
fn recursive_reference_checks_use_declaration_identity_and_allow_direct_calls() {
    check_ok(
        r#"
program T;
procedure WithReference(var Value: integer);
  procedure First();
  begin
    if Value > 0 then
      Value := Value - 1;
      First();
    end if;
    begin
      const First: procedure() := procedure() begin end procedure;
      const Copy: procedure() := procedure() begin First(); end procedure;
    end;
  end procedure;
begin First(); end procedure;
procedure WithValue(Value: integer);
  procedure First();
  begin
    const Copy: procedure() := First;
    const Current: integer := Value;
  end procedure;
begin First(); end procedure;
begin end.
"#,
    );
}

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
