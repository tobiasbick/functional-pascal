//! Local factory returns are legal until a known result crosses a task boundary.

use super::{SEMA_TASK_BOUND_CALLABLE, check_errors, check_ok};

const LOCAL_FACTORY: &str = r#"
function Make(): procedure();
begin
  var Count: integer := 0;
  return procedure() begin Count := Count + 1; end procedure;
end function;
"#;

#[test]
fn local_factory_call_is_legal_but_spawned_known_result_is_rejected() {
    check_ok(&format!(
        "program T; {LOCAL_FACTORY} begin const Work := Make(); Work(); end."
    ));
    let errors = check_errors(&format!(
        "program T; {LOCAL_FACTORY} begin scope discard go Make(); end scope; end."
    ));
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_TASK_BOUND_CALLABLE);
    assert!(errors[0].message.contains("result"));
}

#[test]
fn known_factory_result_remains_local_through_an_ordinary_call_and_send() {
    let errors = check_errors(&format!(
        r#"program T; uses Std.Tasks as Tasks;
{LOCAL_FACTORY}
begin
  const Queue: channel of procedure() := Tasks.CreateChannel(1);
  discard Tasks.Send(Queue, Make());
end."#
    ));
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_TASK_BOUND_CALLABLE);
}

#[test]
fn immutable_aliases_and_closure_workers_keep_their_known_result() {
    let errors = check_errors(&format!(
        r#"program T; {LOCAL_FACTORY}
begin scope
  const Alias := Make;
  const AliasAgain := Alias;
  const Work := function(): procedure()
    begin return Make(); end function;
  discard go AliasAgain();
  discard go Work();
end scope; end."#
    ));
    assert_eq!(errors.len(), 2, "{errors:#?}");
    assert!(
        errors
            .iter()
            .all(|error| error.code == SEMA_TASK_BOUND_CALLABLE)
    );
}

#[test]
fn group_and_supervised_workers_reject_known_local_results() {
    for start in [
        "Tasks.StartTaskInGroup(Group, Work)",
        "Tasks.StartSupervisedTask(Group, Work, 1, 0)",
    ] {
        let errors = check_errors(&format!(
            r#"program T; uses Std.Tasks as Tasks;
{LOCAL_FACTORY}
begin
  const Group := Tasks.CreateTaskGroup();
  const Work := function(Token: Tasks.CancellationToken): procedure()
    begin return Make(); end function;
  discard {start};
end."#
        ));
        assert_eq!(errors.len(), 1, "{start}: {errors:#?}");
        assert_eq!(errors[0].code, SEMA_TASK_BOUND_CALLABLE);
        assert!(errors[0].message.contains("result"));
    }
}

#[test]
fn resolved_methods_and_bound_method_aliases_preserve_result_summaries() {
    let errors = check_errors(
        r#"program T;
type Factory = record
  Seed: integer;
  function Make(Self: Factory): procedure();
  begin
    var Count: integer := Self.Seed;
    return procedure() begin Count := Count + 1; end procedure;
  end function;
end record;
begin scope
  const Boxed := Factory(Seed := 0);
  const Bound := Boxed.Make;
  discard go Boxed.Make();
  discard go (Boxed).Make();
  discard go Bound();
end scope; end."#,
    );
    assert_eq!(errors.len(), 3, "{errors:#?}");
    assert!(
        errors
            .iter()
            .all(|error| error.code == SEMA_TASK_BOUND_CALLABLE)
    );
}

#[test]
fn result_summary_joins_scalar_free_and_local_return_alternatives() {
    let errors = check_errors(&format!(
        r#"program T;
{LOCAL_FACTORY}
function Choose(First: boolean): option of procedure();
begin
  if First then return None; end if;
  return Some(Make());
end function;
begin scope discard go Choose(false); end scope; end."#
    ));
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_TASK_BOUND_CALLABLE);
}

#[test]
fn pure_unknown_recursive_and_forward_results_remain_accepted() {
    check_ok(
        r#"program T;
function Pure(): procedure(); begin return procedure() begin end procedure; end function;
function Unknown(Work: procedure()): procedure(); begin return Work; end function;
function Forward(): procedure(); begin return Later(); end function;
function Later(): procedure(); begin return Pure(); end function;
function Recursive(Again: boolean): procedure();
begin
  if Again then return Recursive(false); end if;
  return Pure();
end function;
begin scope
  discard go Pure();
  discard go Unknown(Pure());
  discard go Forward();
  discard go Recursive(false);
end scope; end."#,
    );
}

#[test]
fn nested_and_shadowed_routines_do_not_share_return_summaries() {
    check_ok(&format!(
        r#"program T;
{LOCAL_FACTORY}
function Outer(): procedure();
  function Make(): procedure();
  begin return procedure() begin end procedure; end function;
begin return Make(); end function;
function Safe(): procedure();
  function Local(): procedure();
  begin
    var Count: integer := 0;
    return procedure() begin Count := Count + 1; end procedure;
  end function;
begin return procedure() begin end procedure; end function;
begin scope discard go Outer(); discard go Safe(); end scope; end."#
    ));
}

#[test]
fn unreachable_local_returns_do_not_change_a_transferable_result() {
    for body in [
        "return procedure() begin end procedure; var Count := 0; return procedure() begin Count := Count + 1; end procedure;",
        "while true do break; var Count := 0; return procedure() begin Count := Count + 1; end procedure; end while; return procedure() begin end procedure;",
        "for I: integer := 0 to 0 do continue; var Count := 0; return procedure() begin Count := Count + 1; end procedure; end for; return procedure() begin end procedure;",
        "if Flag then return procedure() begin end procedure; else return procedure() begin end procedure; end if; var Count := 0; return procedure() begin Count := Count + 1; end procedure;",
        "case Flag of when true: return procedure() begin end procedure; when false: return procedure() begin end procedure; end case; var Count := 0; return procedure() begin Count := Count + 1; end procedure;",
    ] {
        check_ok(&format!(
            "program T; function Safe(Flag: boolean): procedure(); begin {body} end function; begin scope discard go Safe(true); end scope; end."
        ));
    }
}

#[test]
fn unreachable_returns_keep_type_diagnostics_without_adding_transfer_errors() {
    let errors = check_errors(
        "program T; function Safe(): procedure(); begin return procedure() begin end procedure; return 1; end function; begin scope discard go Safe(); end scope; end.",
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("return value")),
        "{errors:#?}"
    );
    assert!(
        errors
            .iter()
            .all(|error| error.code != SEMA_TASK_BOUND_CALLABLE),
        "{errors:#?}"
    );
}

#[test]
fn implicit_try_error_return_rejects_a_known_local_task_result() {
    let errors = check_errors(
        r#"program T;
function Worker(): result of (integer, procedure());
begin
  var Count := 0;
  const Failure: result of (integer, procedure()) :=
    Error(procedure() begin Count := Count + 1; end procedure);
  discard try Failure;
  return Ok(1);
end function;
begin scope discard go Worker(); end scope; end."#,
    );
    assert_eq!(errors.len(), 1, "{errors:#?}");
    assert_eq!(errors[0].code, SEMA_TASK_BOUND_CALLABLE);
    assert!(errors[0].message.contains("result"));
}

#[test]
fn try_success_payloads_do_not_become_error_return_proofs() {
    for (result, declaration, success) in [
        (
            "result of (integer, string)",
            "const Input: result of (procedure(), string) := Ok(Local);",
            "Ok(1)",
        ),
        (
            "option of integer",
            "const Input: option of procedure() := Some(Local);",
            "Some(1)",
        ),
        (
            "result of (integer, procedure())",
            "const Input: result of (procedure(), procedure()) := Ok(Local);",
            "Ok(1)",
        ),
    ] {
        check_ok(&format!(
            r#"program T;
function Worker(): {result};
begin
  var Count := 0;
  const Local := procedure() begin Count := Count + 1; end procedure;
  {declaration}
  discard try Input;
  return {success};
end function;
begin scope discard go Worker(); end scope; end."#
        ));
    }
}

#[test]
fn unreachable_try_error_return_does_not_change_the_result_summary() {
    check_ok(
        r#"program T;
function Worker(): result of (integer, procedure());
begin
  return Ok(1);
  var Count := 0;
  const Failure: result of (integer, procedure()) :=
    Error(procedure() begin Count := Count + 1; end procedure);
  discard try Failure;
end function;
begin scope discard go Worker(); end scope; end."#,
    );
}
