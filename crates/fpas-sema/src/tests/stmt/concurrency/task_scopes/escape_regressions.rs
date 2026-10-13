//! Scope-handle provenance across loops, nested routines, and callable declarations.

use super::{PRELUDE, SEMA_TASK_HANDLE_ESCAPE, program};
use crate::tests::{check_errors, check_ok};

#[test]
fn loop_carried_handles_cannot_reach_outer_storage() {
    for loop_body in [
        "for I: integer := 1 to 2 do B := A; A := Job; end for;",
        "for I: integer in [1, 2] do B := A; A := Job; end for;",
        "var I: integer := 0; while I < 2 do B := A; A := Job; I := I + 1; end while;",
        "var I: integer := 0; repeat B := A; A := Job; I := I + 1; until I = 2;",
    ] {
        let errors = check_errors(&program(&format!(
            "var Outer: task of integer := go Fetch(0);
scope
  var A: task of integer := Outer;
  var B: task of integer := Outer;
  const Job: task of integer := go Fetch(1);
  {loop_body}
  Outer := B;
end scope;"
        )));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TASK_HANDLE_ESCAPE
                    && error.message.contains("storage in `Outer`")),
            "{loop_body}\n{errors:#?}"
        );
    }
}

#[test]
fn nested_routine_captures_contribute_to_parameter_escapes() {
    let declarations = "procedure Leak(Handle: task of integer);
  procedure Save();
  begin Kept.PushInPlace(Handle); end procedure;
begin Save(); end procedure;
function Read(Handle: task of integer): integer;
  function Take(): integer;
  begin return Wait(Handle); end function;
begin return Take(); end function;
";
    for declarations in [declarations.to_string(), declarations.replace("begin Save(); end procedure;",
        "begin const SaveValue: procedure() := Save; SaveValue(); end procedure;"),
        declarations.replace("begin Save(); end procedure;",
        "begin const SaveValues: array of procedure() := [Save]; SaveValues[0](); end procedure;")] {
        let source = format!("{PRELUDE}{declarations}begin scope Leak(go Fetch(1)); end scope; end.");
        let errors = check_errors(&source);
        assert!(errors.iter().any(|error| error.code == SEMA_TASK_HANDLE_ESCAPE
            && error.message.contains("an argument of `Leak`")), "{errors:#?}");
    }
    check_ok(&format!(
        "{PRELUDE}{declarations}begin scope discard Read(go Fetch(1)); end scope; end."
    ));
}

#[test]
fn task_start_callbacks_cannot_retain_scope_handle_parameters() {
    let source = format!(
        "{PRELUDE}
procedure Start(Handle: task of integer);
  procedure Work(Token: CancellationToken); begin discard Wait(Handle); end procedure;
begin
  const Group: TaskGroup := CreateTaskGroup();
  const Child: task := StartTaskInGroup(Group, Work);
  discard CloseTaskGroup(Group);
end procedure;
begin scope Start(go Fetch(1)); end scope; end."
    );
    let errors = check_errors(&source);
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_TASK_HANDLE_ESCAPE),
        "{errors:?}"
    );
}

#[test]
fn higher_order_intrinsics_cannot_pass_owned_handles_to_callable_values() {
    for body in [
        "Jobs.ForEach(Keep);",
        "discard Jobs.Map(function(Handle: task of integer): integer begin Keep(Handle); return 0; end function);",
        "discard Some(Jobs[0]).Map(function(Handle: task of integer): integer begin Keep(Handle); return 0; end function);",
    ] {
        let source = program(&format!(
            "scope const Jobs: array of task of integer := [go Fetch(1)]; {body} end scope;"
        ));
        let errors = check_errors(&source);
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TASK_HANDLE_ESCAPE),
            "{body}\n{errors:#?}"
        );
    }
    check_ok(&program(
        "scope const Values: array of integer := [1];
         discard Values.Map(function(Value: integer): integer begin return Value + 1; end function);
         end scope;",
    ));
}

#[test]
fn local_callable_containers_do_not_escape_handle_parameters() {
    check_ok(&format!(
        "{PRELUDE}
type CallbackBox = record Work: function(): integer; end record;
function Read(Handle: task of integer): integer;
  function Take(): integer; begin return Wait(Handle); end function;
begin
  const Box: CallbackBox := CallbackBox(Work := Take);
  var Callbacks: array of function(): integer := [];
  Callbacks.PushInPlace(Box.Work);
  return Wait(Handle);
end function;
begin scope discard Read(go Fetch(1)); end scope; end."
    ));
}

#[test]
fn declared_closure_initializers_cannot_return_owned_handles() {
    let closure =
        "function(): task of integer begin scope return go Fetch(1); end scope; end function";
    for declaration in [
        format!("const Leak: function(): task of integer := {closure};"),
        format!("var Leak: function(): task of integer := {closure};"),
        format!(
            "procedure Container(); begin const Leak: function(): task of integer := {closure}; null; end procedure;"
        ),
    ] {
        let errors = check_errors(&format!("{PRELUDE}{declaration}begin null; end."));
        assert!(
            errors
                .iter()
                .any(|error| error.code == SEMA_TASK_HANDLE_ESCAPE
                    && error.message.contains("`return`")),
            "{declaration}\n{errors:#?}"
        );
    }
}
