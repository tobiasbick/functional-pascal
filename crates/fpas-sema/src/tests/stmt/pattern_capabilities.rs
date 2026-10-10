//! Pattern bindings preserve static capture proofs and task restrictions.
//!
//! Documentation: `docs/pascal/language/functions/discard.md`

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_TASK_BOUND_CALLABLE, SEMA_UNSAFE_DISCARD};

fn patterns(action: &str) -> Vec<String> {
    vec![
        format!("if Wrapped is Some(const F) then {action} end if;"),
        format!("if false then null; elsif Wrapped is Some(const F) then {action} end if;"),
        format!("while Wrapped is Some(const F) do {action} break; end while;"),
        format!("case Wrapped of when Some(const F): {action} when None: null; end case;"),
        format!("if Ok(Wrapped) is Ok(Some(const F)) then {action} end if;"),
        format!(
            "case Some(Ok(Wrapped)) of when Some(Ok(Some(const F))): {action} when Some(_), None: null; end case;"
        ),
        format!("if Change is const F then {action} end if;"),
    ]
}

fn program(bindings: &str, body: &str) -> String {
    format!(
        "program T; uses Std.Tasks;
function Work(): integer; begin return 42; end function;
function Ready(Job: task): boolean; begin return true; end function;
procedure Main(); begin {bindings} {body} end procedure;
begin Main(); end."
    )
}

const MUTABLE_CAPTURE: &str = "var Count: integer := 0;
const Change: procedure() := procedure() begin Count := Count + 1; end procedure;
const Wrapped: Option of procedure() := Some(Change);";

#[test]
fn pattern_callables_keep_task_bound_flags_in_branches_loops_and_nested_payloads() {
    for body in patterns("go F();") {
        let errors = check_errors(&program(MUTABLE_CAPTURE, &body));
        assert_eq!(errors.len(), 1, "{body}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TASK_BOUND_CALLABLE, "{body}");
    }
}

#[test]
fn pattern_callables_keep_task_bound_flags_in_guards_and_channel_sends() {
    for body in [
        "if Wrapped is Some(const F) and Ready(go F()) then null; end if;",
        "case Wrapped of when Some(const F) if Ready(go F()): null; when Some(_), None: null; end case;",
        "if Wrapped is Some(const F) then const Queue: channel of procedure() := CreateChannel(1); discard Send(Queue, F); end if;",
    ] {
        let errors = check_errors(&program(MUTABLE_CAPTURE, body));
        assert_eq!(errors.len(), 1, "{body}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TASK_BOUND_CALLABLE, "{body}");
    }
}

#[test]
fn closures_capturing_pattern_callables_keep_task_and_discard_metadata() {
    for body in
        patterns("const Alias: procedure() := procedure() begin F(); end procedure; go Alias();")
    {
        let errors = check_errors(&program(MUTABLE_CAPTURE, &body));
        assert_eq!(errors.len(), 1, "{body}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TASK_BOUND_CALLABLE, "{body}");
    }
    for body in patterns(
        "const Alias: function(): integer := function(): integer begin return F(); end function; discard Alias;",
    ) {
        check_ok(&program(
            "const Change: function(): integer := Work;
const Wrapped: Option of function(): integer := Some(Change);",
            &body,
        ));
    }
}

#[test]
fn pattern_callables_keep_known_task_free_capture_proofs() {
    for bindings in [
        "const Change: function(): integer := Work; const Wrapped: Option of function(): integer := Some(Change);",
        "const Base: integer := 40; const Change: function(): integer := function(): integer begin return Base + 2; end function; const Wrapped: Option of function(): integer := Some(Change);",
    ] {
        for body in patterns("discard F;") {
            check_ok(&program(bindings, &body));
        }
        for body in patterns("const Job: task := go F(); const Answer: integer := Wait(Job);") {
            check_ok(&program(bindings, &body));
        }
    }
}

#[test]
fn task_bound_does_not_prevent_discard_of_task_free_pattern_callables() {
    for body in patterns("discard F;") {
        check_ok(&program(MUTABLE_CAPTURE, &body));
    }
}

#[test]
fn unknown_mutable_and_task_capturing_pattern_callables_remain_unsafe_to_discard() {
    for bindings in [
        "const Job: task := go Work(); const Change: function(): integer := function(): integer begin return Wait(Job); end function; const Wrapped: Option of function(): integer := Some(Change);",
        "const Change: function(): integer := Work; var Wrapped: Option of function(): integer := Some(Change);",
    ] {
        for body in patterns("discard F;").into_iter().take(6) {
            let errors = check_errors(&program(bindings, &body));
            assert_eq!(errors.len(), 1, "{body}: {errors:#?}");
            assert_eq!(errors[0].code, SEMA_UNSAFE_DISCARD, "{body}");
        }
    }
    let errors = check_errors(
        "program T; procedure Main(Wrapped: Option of function(): integer);
begin if Wrapped is Some(const F) then discard F; end if; end procedure; begin end.",
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_UNSAFE_DISCARD);
}

#[test]
fn scalar_pattern_bindings_keep_type_based_discard_proofs() {
    check_ok(&program(
        "const Job: task := go Work(); const Wrapped: Result of integer, task of integer := Ok(7);",
        "if Wrapped is Ok(const N) then discard N; end if;
while Wrapped is Ok(const N) do discard N; break; end while;
case Wrapped of when Ok(const N): discard N; when Error(const Job): const N: integer := Wait(Job); end case;
case 7 of when const N if N > 0: discard N; else null; end case;",
    ));
}

#[test]
fn scalar_bindings_do_not_inherit_another_payloads_task_bound_state() {
    for body in [
        "if Mixed is Ok(const N) then const Get: function(): integer := function(): integer begin return N; end function; const Job: task := go Get(); const Answer: integer := Wait(Job); end if;",
        "while Mixed is Ok(const N) do const Get: function(): integer := function(): integer begin return N; end function; const Job: task := go Get(); const Answer: integer := Wait(Job); break; end while;",
        "case Mixed of when Ok(const N): const Get: function(): integer := function(): integer begin return N; end function; const Job: task := go Get(); const Answer: integer := Wait(Job); when Error(_): null; end case;",
    ] {
        check_ok(&program(
            &format!(
                "{MUTABLE_CAPTURE} const Mixed: Result of integer, procedure() := Error(Change);"
            ),
            body,
        ));
    }
}

#[test]
fn generic_scalar_constraints_exclude_callable_capture_capabilities() {
    for constraint in ["Numeric", "Comparable"] {
        check_ok(&format!(
            "program T; uses Std.Tasks;
procedure Main<T: {constraint}>(Value: T);
begin
var Count: integer := 0;
const Change: procedure() := procedure() begin Count := Count + 1; end procedure;
const Mixed: Result of T, procedure() := Error(Change);
if Mixed is Ok(const N) then
  const Get: function(): T := function(): T begin return N; end function;
  const Job: task := go Get();
  const Answer: T := Wait(Job);
end if;
end procedure;
begin Main(42); end."
        ));
    }
}

#[test]
fn extracted_record_callables_keep_capabilities_but_scalar_fields_remain_free() {
    for (body, rejects) in [
        ("go Box.Change();", true),
        ("discard Box.Change;", false),
        (
            "const N: integer := Box.Number; const ReadNumber: function(): integer := function(): integer begin return N; end function; const Job: task := go ReadNumber(); const Answer: integer := Wait(Job);",
            false,
        ),
    ] {
        let source = format!(
            "program T; uses Std.Tasks;
type WorkBox = record Change: procedure(); Number: integer; end record;
procedure Main(); begin
var Count: integer := 0;
const Change: procedure() := procedure() begin Count := Count + 1; end procedure;
const Wrapped: Option of WorkBox := Some(WorkBox(Change := Change, Number := 7));
if Wrapped is Some(const Box) then {body} end if;
end procedure; begin Main(); end."
        );
        if rejects {
            let errors = check_errors(&source);
            assert_eq!(errors.len(), 1, "{errors:#?}");
            assert_eq!(errors[0].code, SEMA_TASK_BOUND_CALLABLE);
        } else {
            check_ok(&source);
        }
    }
}

#[test]
fn binding_names_may_shadow_the_source_without_changing_its_proof() {
    for body in [
        "if Wrapped is Some(const Wrapped) then discard Wrapped; end if;",
        "case Wrapped of when Some(const Wrapped): discard Wrapped; when None: null; end case;",
    ] {
        check_ok(&program(
            "const Wrapped: Option of function(): integer := Some(Work);",
            body,
        ));
    }
}
