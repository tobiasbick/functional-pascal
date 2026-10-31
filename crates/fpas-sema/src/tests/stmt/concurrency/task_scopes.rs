//! Same-routine task ownership, ordinary Result data, and handle escapes.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md`

use super::super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::{SEMA_GO_OUTSIDE_OWNING_SCOPE, SEMA_TASK_HANDLE_ESCAPE};

mod escape_regressions;

#[test]
fn using_resources_allow_borrowing_but_reject_escape() {
    let prefix = "program T; type Resource = record\nrelease procedure Close(Self: Resource); begin null; end procedure;\nend record;\nprocedure Borrow(Value: Resource); begin null; end procedure;\nfunction Keep(Value: Resource): Resource; begin return Value; end function;\n";
    check_ok(&format!(
        "{prefix}begin using const R := Resource(); Borrow(R); end."
    ));
    for body in [
        "using const R := Resource(); discard Keep(R);",
        "var Outer := Resource(); begin using const R := Resource(); Outer := R; end;",
        "using const R := Resource(); using const Other := R;",
    ] {
        let errors = check_errors(&format!("{prefix}begin {body} end."));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("resource owned")),
            "{body}: {errors:#?}"
        );
    }
}

#[test]
fn resource_borrow_cannot_be_forwarded_to_a_spawned_task() {
    let source = "program T; type R = record release procedure Close(Self: R); begin null; end procedure; end record; procedure Borrow(Value: R); begin null; end procedure; procedure Spawn(Value: R); begin scope discard go Borrow(Value); end scope; end procedure; begin using const Value := R(); Spawn(Value); end.";
    let errors = check_errors(source);
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("resource owned")),
        "{errors:?}"
    );
}

#[test]
fn synchronous_callbacks_may_capture_resources_but_escaping_callbacks_may_not() {
    let prelude = "program T; type R = record release procedure Close(Self: R); begin null; end procedure; end record; procedure Borrow(Value: R); begin null; end procedure; procedure Run(Action: procedure()); begin Action(); end procedure; procedure BorrowViaCallback(Value: R); begin [1].ForEach(procedure(Item: integer) begin Borrow(Value); end procedure); end procedure;";
    check_ok(&format!(
        "{prelude} begin using const Value := R(); const Local: procedure() := procedure() begin Borrow(Value); end procedure; const LocalBorrow: procedure(Item: R) := procedure(Item: R) begin Borrow(Item); end procedure; Local(); Run(Local); BorrowViaCallback(Value); [Value].ForEach(Borrow); [Value].ForEach(LocalBorrow); [Value].ForEach(procedure(Item: R) begin Borrow(Item); end procedure); [1].ForEach(procedure(Item: integer) begin Borrow(Value); end procedure); end."
    ));
    let source = format!(
        "{prelude} function Make(): procedure(); begin using const Value := R(); return procedure() begin Borrow(Value); end procedure; end function; begin end."
    );
    let errors = check_errors(&source);
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("resource owned")),
        "{errors:?}"
    );
    let source = format!(
        "{prelude} function Make(): procedure(); begin using const Value := R(); const Callback: procedure() := procedure() begin Borrow(Value); end procedure; return Callback; end function; begin end."
    );
    let errors = check_errors(&source);
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("resource owned")),
        "{errors:?}"
    );
    let source = "program T; type R = record release procedure Close(Self: R); begin null; end procedure; end record; var Kept: array of R := []; procedure Save(Value: R); begin Kept.PushInPlace(Value); end procedure; begin using const Value := R(); [Value].ForEach(Save); end.";
    let errors = check_errors(source);
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("resource owned")),
        "{errors:?}"
    );
    let source = "program T; type R = record release procedure Close(Self: R); begin null; end procedure; end record; var Kept: array of R := []; begin using const Value := R(); [Value].ForEach(procedure(Item: R) begin Kept.PushInPlace(Item); end procedure); end.";
    let errors = check_errors(source);
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("resource owned")),
        "{errors:?}"
    );
}

#[test]
fn callback_cannot_hide_parameter_escape_from_its_callers() {
    let source = "program T; type R = record release procedure Close(Self: R); begin null; end procedure; end record; var Kept: array of R := []; procedure Leak(Value: R); begin const Save: procedure() := procedure() begin Kept.PushInPlace(Value); end procedure; Save(); end procedure; begin using const Value := R(); Leak(Value); end.";
    let errors = check_errors(source);
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("resource owned")),
        "{errors:?}"
    );
}

#[test]
fn using_requires_a_valid_release_contract() {
    let errors = check_errors("program T; begin using const Value := 1; end.");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("no resource release contract")),
        "{errors:?}"
    );
    for members in [
        "release procedure Close(Self: R; Extra: integer); begin null; end procedure;",
        "release procedure A(Self: R); begin null; end procedure; release procedure B(Self: R); begin null; end procedure;",
    ] {
        let errors = check_errors(&format!(
            "program T; type R = record {members} end record; begin end."
        ));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("Invalid release procedure")),
            "{errors:?}"
        );
    }
    let (_, errors) = fpas_parser::parse(
        "program T; type R = record release procedure Close(var Self: R); begin null; end procedure; end record; begin end.",
    );
    assert!(!errors.is_empty(), "release receiver must be read-only");
}

#[test]
fn instantiated_generic_records_keep_their_release_contract() {
    check_ok(
        "program T; type Box of T = record Value: T; release procedure Close(Self: Box of T); begin null; end procedure; end record; begin using const B: Box of integer := Box(Value := 1); end.",
    );
}

#[test]
fn standard_resource_release_has_its_checked_native_signature() {
    let (program, parse_errors) = fpas_parser::parse(
        "program T; uses Std.Io.Fs as Fs; begin using const F := Fs.Open('test', Fs.FileMode.Read).Unwrap(); end.",
    );
    assert!(parse_errors.is_empty(), "{parse_errors:?}");
    let metadata = crate::analyze_with_types(&program);
    assert!(metadata.errors.is_empty(), "{:?}", metadata.errors);
    let release = metadata.resource_releases.values().next().expect("release");
    assert!(release.native);
    assert_eq!(release.name, "Std.Io.Fs.Close");
    assert!(
        matches!(&release.signature, crate::Ty::Function(function) if function.params.len() == 1)
    );
}

#[test]
fn publishing_a_borrowed_server_creates_an_independently_owned_endpoint() {
    check_ok(
        "program T; uses Std.Ai.Mcp as Mcp; begin using const Server := Mcp.Create(Mcp.DefaultLimits()).Unwrap(); using const Endpoint := Mcp.Publish(Server, Mcp.DefaultHttpOptions()).Unwrap(); end.",
    );
    let errors = check_errors(
        "program T; uses Std.Ai.Mcp as Mcp; function Make(): Mcp.Endpoint; begin using const Server := Mcp.Create(Mcp.DefaultLimits()).Unwrap(); return Mcp.Publish(Server, Mcp.DefaultHttpOptions()).Unwrap(); end function; begin end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("resource owned")),
        "{errors:?}"
    );
    for source in [
        "program T; uses Std.Ai.Mcp as Mcp; var Kept: option of Mcp.Endpoint := None; begin begin using const Server := Mcp.Create(Mcp.DefaultLimits()).Unwrap(); Kept := Some(Mcp.Publish(Server, Mcp.DefaultHttpOptions()).Unwrap()); end; end.",
        "program T; uses Std.Ai.Mcp as Mcp, Std.Tasks as Tasks; begin const C: channel of Mcp.Endpoint := Tasks.CreateChannel(1); using const Server := Mcp.Create(Mcp.DefaultLimits()).Unwrap(); const Endpoint := Mcp.Publish(Server, Mcp.DefaultHttpOptions()).Unwrap(); discard Tasks.Send(C, Endpoint); end.",
        "program T; uses Std.Ai.Mcp as Mcp; function Make(): procedure(); begin using const Server := Mcp.Create(Mcp.DefaultLimits()).Unwrap(); const Endpoint := Mcp.Publish(Server, Mcp.DefaultHttpOptions()).Unwrap(); return procedure() begin discard Mcp.Access(Endpoint); end procedure; end function; begin end.",
    ] {
        let errors = check_errors(source);
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("resource owned")),
            "{errors:?}"
        );
    }
}

#[test]
fn accepting_a_borrowed_listener_creates_an_independently_owned_connection() {
    check_ok(
        "program T; uses Std.Net.Sockets as Net, Std.Tasks as Tasks; procedure Work(Token: Tasks.CancellationToken); begin using const Listener := Net.Listen('127.0.0.1', 0).Unwrap(); using const First := Net.Accept(Listener).Unwrap(); using const Second := Net.AcceptWithCancellation(Listener, Token).Unwrap(); end procedure; begin end.",
    );
}

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
