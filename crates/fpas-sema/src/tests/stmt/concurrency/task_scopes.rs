//! Task scope ownership, lost errors, and handle escapes.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md`

use super::super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{
    SEMA_GO_OUTSIDE_OWNING_SCOPE, SEMA_LOST_TASK_ERROR, SEMA_TASK_HANDLE_ESCAPE,
};

mod escape_regressions;

const PRELUDE: &str = "program T;
uses Std.Tasks;
var Kept: array of task of integer := [];
function Fetch(Value: integer): integer;
begin
  return Value;
end function;
function Load(Value: integer): result of (integer, string);
begin
  return Ok(Value);
end function;
procedure Each(Values: array of integer; Action: procedure(Value: integer));
begin
  for Value: integer in Values do
    Action(Value);
  end for;
end procedure;
function Total(Handles: array of task of integer): integer;
begin
  var Sum: integer := 0;
  for Handle: task of integer in Handles do
    Sum := Sum + Wait(Handle);
  end for;
  return Sum;
end function;
function TotalTwice(Handles: array of task of integer): integer;
begin
  return Total(Handles) + Total(Handles);
end function;
procedure Keep(Handle: task of integer);
begin
  Kept.Push(Handle);
end procedure;
procedure KeepLater(Handle: task of integer);
begin
  Keep(Handle);
end procedure;
function First(Handles: array of task of integer): task of integer;
begin
  return Handles[0];
end function;
procedure Replace(var Slot: task of integer; Handle: task of integer);
begin
  Slot := Handle;
end procedure;
";

fn program(body: &str) -> String {
    format!("{PRELUDE}begin\n{body}\nend.")
}

fn assert_code(body: &str, code: fpas_diagnostics::DiagnosticCode) {
    let errors = check_errors(&program(body));
    assert!(
        errors.iter().any(|error| error.code == code),
        "{body}\n{errors:#?}"
    );
}

#[test]
fn go_in_anonymous_routine_inside_scope_is_rejected_unless_it_opens_a_scope() {
    assert_code(
        "scope
  Each([1], procedure(Value: integer) begin
    go Fetch(Value);
  end procedure);
end scope;",
        SEMA_GO_OUTSIDE_OWNING_SCOPE,
    );
    check_ok(&program(
        "scope
  Each([1], procedure(Value: integer) begin
    scope
      go Fetch(Value);
    end scope;
  end procedure);
end scope;
Each([2], procedure(Value: integer) begin
  go Fetch(Value);
end procedure);",
    ));
}

#[test]
fn go_statement_inside_scope_must_not_drop_a_result_error() {
    assert_code(
        "scope
  go Load(1);
end scope;",
        SEMA_LOST_TASK_ERROR,
    );
    check_ok(&program(
        "go Load(1);
scope
  const Job: task of result of (integer, string) := go Load(2);
  case Wait(Job) of
    when Ok(_): null;
    when Error(_): null;
  end case;
end scope;",
    ));
}

#[test]
fn scope_owned_handles_are_rejected_at_every_escape_route() {
    for (body, route) in [
        (
            "var Outer: task of integer := go Fetch(0);
scope
  Outer := go Fetch(1);
end scope;",
            "storage in `Outer`",
        ),
        (
            "var Outer: array of task of integer := [];
scope
  Outer.Push(go Fetch(1));
end scope;",
            "storage in `Outer`",
        ),
        (
            "var Outer: task of integer := go Fetch(0);
scope
  Replace(var Outer, go Fetch(1));
end scope;",
            "an argument of `Replace`",
        ),
        (
            "scope
  Keep(go Fetch(1));
end scope;",
            "an argument of `Keep`",
        ),
        (
            "scope
  KeepLater(go Fetch(1));
end scope;",
            "an argument of `KeepLater`",
        ),
        (
            "scope
  const Job: task of integer := go Fetch(1);
  const Queue: channel of task of integer := CreateChannel(1);
  discard Send(Queue, Job);
end scope;",
            "a channel send",
        ),
        (
            "scope
  const Job: task of integer := go Fetch(1);
  const Later: function(): integer := function(): integer begin
    return Wait(Job);
  end function;
end scope;",
            "capture of `Job`",
        ),
        (
            "const Accept: procedure(Handle: task of integer) := Keep;
scope
  Accept(go Fetch(1));
end scope;",
            "callable value",
        ),
        (
            "scope
  const Job: task of integer := First([go Fetch(1)]);
end scope;",
            "an argument of `First`",
        ),
        (
            "scope
  var Inner: task of integer := go Fetch(0);
  scope
    Inner := go Fetch(1);
  end scope;
end scope;",
            "storage in `Inner`",
        ),
    ] {
        let errors = check_errors(&program(body));
        assert!(
            errors.iter().any(|error| error.code == SEMA_TASK_HANDLE_ESCAPE
                && error.message.contains(route)),
            "{body}\n{errors:#?}"
        );
    }
}

#[test]
fn returning_a_scope_owned_handle_is_rejected() {
    let errors = check_errors(&format!(
        "{PRELUDE}function Leak(): task of integer;
begin
  scope
    return go Fetch(1);
  end scope;
end function;
begin
end."
    ));
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_TASK_HANDLE_ESCAPE
                && error.message.contains("through `return`")
                && error.message.contains("line ")),
        "{errors:#?}"
    );
}

#[test]
fn scope_owned_handles_stay_usable_inside_their_scope() {
    check_ok(&program(
        "scope
  var Jobs: array of task of integer := [];
  Jobs.Push(go Fetch(1));
  Jobs.Push(go Fetch(2));
  const Pair: array of task of integer := [Jobs[0], Jobs[1]];
  discard Total(Pair);
  discard TotalTwice(Jobs);
  WaitAll(Jobs);
  discard WaitAny(Jobs);
  const Sibling: task of integer := go Total(Jobs);
  discard Wait(Sibling);
  scope
    discard Wait(Jobs[0]);
    const Inner: task of integer := go Fetch(3);
    discard Wait(Inner);
  end scope;
  const Count: integer := Wait(Jobs[1]);
end scope;",
    ));
}
