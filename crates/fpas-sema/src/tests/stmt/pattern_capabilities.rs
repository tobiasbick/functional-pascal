//! Pattern bindings preserve cross-task restrictions; discard accepts every value.
//!
//! Documentation: `docs/pascal/language/functions/discard.md`

use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE;

fn patterns(action: &str) -> Vec<String> {
    vec![
        format!("case Wrapped of when Some(const F): {action} when None: null; end case;"),
        format!(
            "while true do case Wrapped of when Some(const F): {action} break; when None: break; end case; end while;"
        ),
        format!(
            "case Some(Ok(Wrapped)) of when Some(Ok(Some(const F))): {action} when Some(_), None: null; end case;"
        ),
    ]
}

fn program(bindings: &str, body: &str) -> String {
    let bindings = bindings.replace("const Job: task of integer := go Work();", "const Group := CreateTaskGroup(); const Job := StartTaskInGroup(Group, function(Token: CancellationToken): integer begin return Work(); end function);");
    format!(
        "program T; uses Std.Tasks;
function Work(): integer; begin return 42; end function;
function Ready(Job: task): boolean; begin return true; end function;
procedure Main(); begin scope {bindings} {body} end scope; end procedure;
begin Main(); end."
    )
}

const MUTABLE_CAPTURE: &str = "var Count: integer := 0;
const Change: procedure() := procedure() begin Count := Count + 1; end procedure;
const Wrapped: Option of procedure() := Some(Change);";

#[test]
fn named_payload_constructors_preserve_task_bound_callables() {
    for (ty, constructor, other) in [
        ("option of function(): integer", "Some", "None"),
        (
            "result of (function(): integer, string)",
            "Ok",
            "Error(Value := _)",
        ),
        (
            "result of (integer, function(): integer)",
            "Error",
            "Ok(Value := _)",
        ),
    ] {
        check_ok(&program(
            &format!(
                "const Wrapped: {ty} := {constructor}(Value := function(): integer begin return 1; end function);"
            ),
            "discard Wrapped;",
        ));
        let errors = check_errors(&program(
            &format!(
                "var Count: integer := 0; const Wrapped: {ty} := {constructor}(Value := function(): integer begin Count := Count + 1; return Count; end function);"
            ),
            &format!(
                "case Wrapped of when {constructor}(Value := const F): const Job := go F(); discard Wait(Job); when {other}: null; end case;"
            ),
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TASK_BOUND_CALLABLE),
            "{constructor}: {errors:#?}"
        );
    }
}

#[test]
fn pattern_callables_keep_task_bound_flags_in_branches_loops_and_nested_payloads() {
    for body in patterns("discard go F();") {
        let errors = check_errors(&program(MUTABLE_CAPTURE, &body));
        assert_eq!(errors.len(), 1, "{body}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TASK_BOUND_CALLABLE, "{body}");
    }
}

#[test]
fn pattern_callables_keep_task_bound_flags_in_guards_and_channel_sends() {
    for body in [
        "case Wrapped of when Some(const F) if Ready(go F()): null; when Some(_), None: null; end case;",
        "case Wrapped of when Some(const F): const Queue: channel of procedure() := CreateChannel(1); discard Send(Queue, F); when None: null; end case;",
    ] {
        let errors = check_errors(&program(MUTABLE_CAPTURE, body));
        assert_eq!(errors.len(), 1, "{body}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TASK_BOUND_CALLABLE, "{body}");
    }
}

#[test]
fn closures_capturing_pattern_callables_keep_task_bound_metadata() {
    for body in patterns(
        "const Alias: procedure() := procedure() begin F(); end procedure; discard go Alias();",
    ) {
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
fn immutable_pattern_callables_can_be_spawned_and_discarded() {
    for bindings in [
        "const Change: function(): integer := Work; const Wrapped: Option of function(): integer := Some(Change);",
        "const Base: integer := 40; const Change: function(): integer := function(): integer begin return Base + 2; end function; const Wrapped: Option of function(): integer := Some(Change);",
    ] {
        for body in patterns("discard F;") {
            check_ok(&program(bindings, &body));
        }
        for body in
            patterns("const Job: task of integer := go F(); const Answer: integer := Wait(Job);")
        {
            check_ok(&program(bindings, &body));
        }
    }
}

#[test]
fn task_bound_does_not_prevent_discard_of_pattern_callables() {
    for body in patterns("discard F;") {
        check_ok(&program(MUTABLE_CAPTURE, &body));
    }
}

#[test]
fn pattern_callable_discard_is_independent_of_captures_and_mutability() {
    for bindings in [
        "const Job: task of integer := go Work(); const Change: function(): integer := function(): integer begin return Wait(Job); end function; const Wrapped: Option of function(): integer := Some(Change);",
        "const Change: function(): integer := Work; var Wrapped: Option of function(): integer := Some(Change);",
    ] {
        for body in patterns("discard F;") {
            check_ok(&program(bindings, &body));
        }
    }
    check_ok("program T; procedure Main(Wrapped: Option of function(): integer);
        begin case Wrapped of when Some(const F): discard F; when None: null; end case; end procedure; begin end.");
}

#[test]
fn scalar_pattern_bindings_accept_ordinary_discard() {
    check_ok(&program(
        "const Job: task of integer := go Work(); const Wrapped: Result of (integer, task of integer) := Ok(7);",
        "case Wrapped of when Ok(const N): discard N; when Error(const Job): const N: integer := Wait(Job); end case;
case 7 of when const N if N > 0: discard N; else null; end case;",
    ));
}

#[test]
fn scalar_bindings_do_not_inherit_another_payloads_task_bound_state() {
    for body in [
        "while true do case Mixed of when Ok(const N): const Get: function(): integer := function(): integer begin return N; end function; const Job: task of integer := go Get(); const Answer: integer := Wait(Job); break; when Error(_): break; end case; end while;",
        "case Mixed of when Ok(const N): const Get: function(): integer := function(): integer begin return N; end function; const Job: task of integer := go Get(); const Answer: integer := Wait(Job); when Error(_): null; end case;",
    ] {
        check_ok(&program(
            &format!(
                "{MUTABLE_CAPTURE} const Mixed: Result of (integer, procedure()) := Error(Change);"
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
begin scope
var Count: integer := 0;
const Change: procedure() := procedure() begin Count := Count + 1; end procedure;
const Mixed: Result of (T, procedure()) := Error(Change);
case Mixed of when Ok(const N):
  const Get: function(): T := function(): T begin return N; end function;
  const Job: task of T := go Get();
  const Answer: T := Wait(Job);
when Error(_): null; end case;
 end scope; end procedure;
begin Main(42); end."
        ));
    }
}

#[test]
fn extracted_record_callables_keep_capabilities_but_scalar_fields_remain_free() {
    for (body, rejects) in [
        ("discard go Box.Change();", true),
        ("discard Box.Change;", false),
        (
            "const N: integer := Box.Number; const ReadNumber: function(): integer := function(): integer begin return N; end function; const Job: task of integer := go ReadNumber(); const Answer: integer := Wait(Job);",
            false,
        ),
    ] {
        let source = format!(
            "program T; uses Std.Tasks;
type WorkBox = record Change: procedure(); Number: integer; end record;
procedure Main(); begin scope
var Count: integer := 0;
const Change: procedure() := procedure() begin Count := Count + 1; end procedure;
const Wrapped: Option of WorkBox := Some(WorkBox(Change := Change, Number := 7));
case Wrapped of when Some(const Box): {body} when None: null; end case;
end scope; end procedure; begin Main(); end."
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
    for body in
        ["case Wrapped of when Some(const Wrapped): discard Wrapped; when None: null; end case;"]
    {
        check_ok(&program(
            "const Wrapped: Option of function(): integer := Some(Work);",
            body,
        ));
    }
}
