//! Task scope completion, cancellation, and failure propagation through the VM.
//!
//! **Documentation:** `docs/pascal/language/concurrency/task-scopes.md`

use super::*;
use fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC;

const PRELUDE: &str = "uses Std.Tasks as Tasks, Std.Time as Time;\nprocedure Produce(Queue: channel of integer; Value: integer);\nbegin\n  Time.Sleep(5);\n  discard Tasks.Send(Queue, Value);\nend procedure;\nfunction Block(Gate: channel of integer): integer;\nbegin\n  return Tasks.Receive(Gate).Unwrap();\nend function;\nfunction Spin(): integer;\nbegin\n  var Count: integer := 0;\n  while true do\n    Count := Count + 1;\n  end while;\n  return Count;\nend function;\nfunction Fail(Message: string): integer;\nbegin\n  Time.Sleep(5);\n  panic(Message);\n  return 0;\nend function;\n";

fn program(declarations: &str, body: &str) -> String {
    format!("program Scopes;\n{PRELUDE}{declarations}begin\n{body}\nend.")
}

#[test]
fn normal_exit_waits_for_both_go_forms() {
    assert_succeeds(&program(
        "",
        "const Queue: channel of integer := Tasks.CreateChannel(2);\nscope\n  discard go Produce(Queue, 1);\n  const Second: task := go Produce(Queue, 2);\nend scope;\nfor Index: integer := 1 to 2 do\n  case Tasks.TryReceive(Queue) of\n    when Ok(Some(_)): null;\n    when Ok(None): panic('scope ended before its children');\n    when Error(_): panic('closed');\n  end case;\nend for;",
    ));
}

#[test]
fn early_exits_cancel_running_and_blocked_children() {
    assert_succeeds(&program(
        "function FirstPositive(Values: array of integer): integer;\nbegin\n  for Value: integer in Values do\n    scope\n      discard go Spin();\n      if Value < 0 then\n        continue;\n      end if;\n      if Value = 0 then\n        break;\n      end if;\n      return Value;\n    end scope;\n  end for;\n  return 0;\nend function;\nfunction Load(): result of (integer, string);\nbegin\n  return Error('missing');\nend function;\nfunction Forward(Gate: channel of integer): result of (integer, string);\nbegin\n  scope\n    const Blocked: task of integer := go Block(Gate);\n    const Failed: task of result of (integer, string) := go Load();\n    const Value: integer := try Tasks.Wait(Failed);\n    return Ok(Value + Tasks.Wait(Blocked));\n  end scope;\nend function;\n",
        "if FirstPositive([-1, 4]) <> 4 then panic('return value'); end if;\nif FirstPositive([0, 4]) <> 0 then panic('break value'); end if;\nconst Gate: channel of integer := Tasks.CreateChannel(1);\ncase Forward(Gate) of\n  when Ok(_): panic('error was not forwarded');\n  when Error(const Message): if Message <> 'missing' then panic(Message); end if;\nend case;",
    ));
}

#[test]
fn helpers_use_an_explicit_group_for_work_that_outlives_the_callers_scope() {
    assert_succeeds(&program(
        "procedure Relay(Gate: channel of integer; Queue: channel of integer);\nbegin\n  discard Tasks.Send(Queue, Block(Gate));\nend procedure;\nprocedure StartProducer(Group: Tasks.TaskGroup; Gate: channel of integer; Queue: channel of integer);\nbegin\n  discard Tasks.StartTaskInGroup(Group, procedure(Token: Tasks.CancellationToken) begin Relay(Gate, Queue); end procedure);\nend procedure;\n",
        "const Gate: channel of integer := Tasks.CreateChannel(1);\nconst Queue: channel of integer := Tasks.CreateChannel(1);\nconst Group := Tasks.CreateTaskGroup();\nscope\n  StartProducer(Group, Gate, Queue);\nend scope;\ndiscard Tasks.Send(Gate, 7);\nif Tasks.Receive(Queue).Unwrap() <> 7 then panic('group task result'); end if;\ndiscard Tasks.CloseTaskGroup(Group);",
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
        "scope\n  const Job: task of result of (integer, string) := go Load();\n  Tasks.WaitAll([Job]);\nend scope;",
    ));
}

#[test]
fn child_failure_cancels_siblings_and_body_and_is_reraised() {
    for body in [
        // The body waits on a channel that no task will ever fill.
        "const Gate: channel of integer := Tasks.CreateChannel(1);\nscope\n  discard go Spin();\n  const Failed: task of integer := go Fail('child failed');\n  discard Block(Gate);\nend scope;",
        // The body waits for the failed child itself.
        "scope\n  const Failed: task of integer := go Fail('child failed');\n  discard Tasks.Wait(Failed);\nend scope;",
        // A failure in a nested scope leaves the outer scope too.
        "scope\n  discard go Spin();\n  scope\n    discard go Fail('child failed');\n    Time.Sleep(5000);\n  end scope;\nend scope;",
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
