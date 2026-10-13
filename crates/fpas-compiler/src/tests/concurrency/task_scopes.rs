//! Task scope completion, cancellation, and failure propagation through the VM.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md`

use super::*;
use fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC;

const PRELUDE: &str = "uses Std.Tasks, Std.Time;
procedure Produce(Queue: channel of integer; Value: integer);
begin
  Sleep(5);
  discard Send(Queue, Value);
end procedure;
function Block(Gate: channel of integer): integer;
begin
  return Receive(Gate).Unwrap();
end function;
function Spin(): integer;
begin
  var Count: integer := 0;
  while true do
    Count := Count + 1;
  end while;
  return Count;
end function;
function Fail(Message: string): integer;
begin
  Sleep(5);
  panic(Message);
  return 0;
end function;
";

fn program(declarations: &str, body: &str) -> String {
    format!("program Scopes;\n{PRELUDE}{declarations}begin\n{body}\nend.")
}

#[test]
fn normal_exit_waits_for_both_go_forms() {
    assert_succeeds(&program(
        "",
        "const Queue: channel of integer := CreateChannel(2);
scope
  discard go Produce(Queue, 1);
  const Second: task := go Produce(Queue, 2);
end scope;
for Index: integer := 1 to 2 do
  case TryReceive(Queue) of
    when Ok(Some(_)): null;
    when Ok(None): panic('scope ended before its children');
    when Error(_): panic('closed');
  end case;
end for;",
    ));
}

#[test]
fn early_exits_cancel_running_and_blocked_children() {
    assert_succeeds(&program(
        "function FirstPositive(Values: array of integer): integer;
begin
  for Value: integer in Values do
    scope
      discard go Spin();
      if Value < 0 then
        continue;
      end if;
      if Value = 0 then
        break;
      end if;
      return Value;
    end scope;
  end for;
  return 0;
end function;
function Load(): result of (integer, string);
begin
  return Error('missing');
end function;
function Forward(Gate: channel of integer): result of (integer, string);
begin
  scope
    const Blocked: task of integer := go Block(Gate);
    const Failed: task of result of (integer, string) := go Load();
    const Value: integer := try Wait(Failed);
    return Ok(Value + Wait(Blocked));
  end scope;
end function;
",
        "if FirstPositive([-1, 4]) <> 4 then panic('return value'); end if;
if FirstPositive([0, 4]) <> 0 then panic('break value'); end if;
const Gate: channel of integer := CreateChannel(1);
case Forward(Gate) of
  when Ok(_): panic('error was not forwarded');
  when Error(const Message): if Message <> 'missing' then panic(Message); end if;
end case;",
    ));
}

#[test]
fn helpers_use_an_explicit_group_for_work_that_outlives_the_callers_scope() {
    assert_succeeds(&program(
        "procedure Relay(Gate: channel of integer; Queue: channel of integer);
begin
  discard Send(Queue, Block(Gate));
end procedure;
procedure StartProducer(Group: TaskGroup; Gate: channel of integer; Queue: channel of integer);
begin
  discard StartTaskInGroup(Group, procedure(Token: CancellationToken) begin Relay(Gate, Queue); end procedure);
end procedure;
",
        "const Gate: channel of integer := CreateChannel(1);
const Queue: channel of integer := CreateChannel(1);
const Group := CreateTaskGroup();
scope
  StartProducer(Group, Gate, Queue);
end scope;
discard Send(Gate, 7);
if Receive(Queue).Unwrap() <> 7 then panic('group task result'); end if;
discard CloseTaskGroup(Group);",
    ));
}

#[test]
fn a_returned_error_does_not_fail_the_scope() {
    assert_succeeds(&program(
        "function Load(): result of (integer, string);
begin
  return Error('ignored');
end function;
",
        "scope
  const Job: task of result of (integer, string) := go Load();
  WaitAll([Job]);
end scope;",
    ));
}

#[test]
fn child_failure_cancels_siblings_and_body_and_is_reraised() {
    for body in [
        // The body waits on a channel that no task will ever fill.
        "const Gate: channel of integer := CreateChannel(1);
scope
  discard go Spin();
  const Failed: task of integer := go Fail('child failed');
  discard Block(Gate);
end scope;",
        // The body waits for the failed child itself.
        "scope
  const Failed: task of integer := go Fail('child failed');
  discard Wait(Failed);
end scope;",
        // A failure in a nested scope leaves the outer scope too.
        "scope
  discard go Spin();
  scope
    discard go Fail('child failed');
    Sleep(5000);
  end scope;
end scope;",
    ] {
        let error = run_program(&program("", body)).expect_err("scope failure");
        assert_eq!(error.code, RUNTIME_PROGRAM_PANIC, "{body}\n{error:?}");
        assert!(error.message.contains("child failed"), "{error:?}");
        assert!(
            !error.help.clone().unwrap_or_default().contains("Further"),
            "a cancellation is not a further failure: {error:?}"
        );
    }
}

#[test]
fn a_body_panic_cancels_and_waits_for_children() {
    let error = run_program(&program(
        "",
        "scope
  discard go Spin();
  panic('body failed');
end scope;",
    ))
    .expect_err("body panic");
    assert!(error.message.contains("body failed"), "{error:?}");
}
