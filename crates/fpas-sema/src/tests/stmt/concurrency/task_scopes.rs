//! Same-routine task ownership, ordinary Result data, and handle escapes.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md`

use super::super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_GO_OUTSIDE_OWNING_SCOPE, SEMA_TASK_HANDLE_ESCAPE};

mod escape_regressions;

const PRELUDE: &str = "program T;\nuses Std.Tasks as Tasks;\nvar Kept: array of task of integer := [];\nfunction Fetch(Value: integer): integer;\nbegin\n  return Value;\nend function;\nfunction Load(Value: integer): result of (integer, string);\nbegin\n  return Ok(Value);\nend function;\nprocedure Each(Values: array of integer; Action: procedure(Value: integer));\nbegin\n  for Value: integer in Values do\n    Action(Value);\n  end for;\nend procedure;\nfunction Total(Handles: array of task of integer): integer;\nbegin\n  var Sum: integer := 0;\n  for Handle: task of integer in Handles do\n    Sum := Sum + Tasks.Wait(Handle);\n  end for;\n  return Sum;\nend function;\nfunction TotalTwice(Handles: array of task of integer): integer;\nbegin\n  return Total(Handles) + Total(Handles);\nend function;\nprocedure Keep(Handle: task of integer);\nbegin\n  Kept.PushInPlace(Handle);\nend procedure;\nprocedure KeepLater(Handle: task of integer);\nbegin\n  Keep(Handle);\nend procedure;\nfunction First(Handles: array of task of integer): task of integer;\nbegin\n  return Handles[0];\nend function;\nprocedure Replace(var Slot: task of integer; Handle: task of integer);\nbegin\n  Slot := Handle;\nend procedure;\n";

fn program(body: &str) -> String {
    format!("{PRELUDE}begin scope\n{body}\nend scope; end.")
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
    discard go Fetch(Value);
  end procedure);
end scope;",
        SEMA_GO_OUTSIDE_OWNING_SCOPE,
    );
    check_ok(&program(
        "scope
  Each([1], procedure(Value: integer) begin
    scope
      discard go Fetch(Value);
    end scope;
  end procedure);
end scope;
Each([2], procedure(Value: integer) begin scope
  discard go Fetch(Value);
 end scope; end procedure);",
    ));
}

#[test]
fn discarded_go_may_return_an_ordinary_result_error() {
    check_ok(&program(
        "scope
  discard go Load(1);
end scope;",
    ));
    check_ok(&program(
        "discard go Load(1);\nscope\n  const Job: task of result of (integer, string) := go Load(2);\n  case Tasks.Wait(Job) of\n    when Ok(_): null;\n    when Error(_): null;\n  end case;\nend scope;",
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
  Outer.PushInPlace(go Fetch(1));
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
            "scope\n  const Job: task of integer := go Fetch(1);\n  const Queue: channel of task of integer := Tasks.CreateChannel(1);\n  discard Tasks.Send(Queue, Job);\nend scope;",
            "a channel send",
        ),
        (
            "scope\n  const Job: task of integer := go Fetch(1);\n  const Later: function(): integer := function(): integer begin\n    return Tasks.Wait(Job);\n  end function;\nend scope;",
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
        "scope\n  var Jobs: array of task of integer := [];\n  Jobs.PushInPlace(go Fetch(1));\n  Jobs.PushInPlace(go Fetch(2));\n  const Pair: array of task of integer := [Jobs[0], Jobs[1]];\n  discard Total(Pair);\n  discard TotalTwice(Jobs);\n  Tasks.WaitAll(Jobs);\n  discard Tasks.WaitAny(Jobs);\n  const Sibling: task of integer := go Total(Jobs);\n  discard Tasks.Wait(Sibling);\n  scope\n    discard Tasks.Wait(Jobs[0]);\n    const Inner: task of integer := go Fetch(3);\n    discard Tasks.Wait(Inner);\n  end scope;\n  const Count: integer := Tasks.Wait(Jobs[1]);\nend scope;",
    ));
}
