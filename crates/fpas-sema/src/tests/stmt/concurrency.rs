use super::super::{check_errors, check_ok};
use fpas_diagnostics::codes::SEMA_TASK_BOUND_CALLABLE;
use fpas_parser::{ParseDiagnostic, parse};

mod groups;
mod task_scopes;
mod task_types;
mod worker_state;

#[test]
fn selection_send_rejects_task_bound_values() {
    let errors = check_errors(
        r#"program T;
uses Std.Tasks as Tasks;
begin
  var Count: integer := 0;
  const Work: procedure() := procedure() begin Count := Count + 1; end procedure;
  const Q: channel of procedure() := Tasks.CreateChannel(1);
  Tasks.SendCase(Q, Work, procedure(R: result of (boolean, Tasks.TaskError)) begin end procedure);
end."#,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_TASK_BOUND_CALLABLE),
        "{errors:?}"
    );
}

#[test]
fn selection_cases_reject_wrong_payload_callback_and_control_types() {
    for call in [
        "ReceiveCase(Q, procedure(R: result of (string, string)) begin end procedure)",
        "SendCase(Q, 'wrong', procedure(R: result of (boolean, string)) begin end procedure)",
        "SendCase(Q, 1, procedure(R: result of (integer, string)) begin end procedure)",
        "TimerCase(0, function(): integer begin return 1; end function)",
        "TimerCase('wrong', procedure() begin end procedure)",
        "TaskCase(1, procedure() begin end procedure)",
        "CancellationCase(CreateCancellationSource(), procedure() begin end procedure)",
        "Select([1])",
        "CloseWaitCase(CreateCancellationSource())",
    ] {
        let source = format!(
            "program T; uses Std.Tasks as Tasks; begin const Q: channel of integer := Tasks.CreateChannel(1); {call}; end."
        );
        assert!(
            !check_errors(&source).is_empty(),
            "accepted invalid selection: {call}"
        );
    }
}

#[test]
fn controlled_wait_any_checks_control_types_and_arity() {
    for call in [
        "WaitAnyWithTimeout(Tasks, 'bad')",
        "WaitAnyWithCancellation(Tasks, CreateCancellationSource())",
        "WaitAnyWithTimeout(Tasks)",
        "WaitAnyWithCancellation([1, 2], GetCancellationToken(CreateCancellationSource()))",
    ] {
        let source = format!(
            "program T; uses Std.Tasks as StdTasks; begin const Tasks: array of task := []; {call}; end."
        );
        assert!(!check_errors(&source).is_empty(), "{call}");
    }
}

#[test]
fn wait_any_rejects_non_task_arrays_and_reports_its_own_arity() {
    let errors =
        check_errors("program T; uses Std.Tasks as Tasks; begin Tasks.WaitAny([1, 2]); end.");
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("expected `array of task`"))
    );
    let errors = check_errors("program T; uses Std.Tasks as Tasks; begin Tasks.WaitAny(); end.");
    assert!(errors.iter().any(|error| error.message.contains("WaitAny")));
}

#[test]
fn network_io_cancellation_requires_a_token_not_a_source() {
    let errors = check_errors(
        "program T;\nuses Std.Net as Net, Std.Tasks as Tasks;\nprocedure Invalid(ConnectionValue: Net.Connection);\nbegin\n  Net.ReceiveBytesWithCancellation(ConnectionValue, 1, Tasks.CreateCancellationSource());\n  Net.SendBytesWithCancellation(ConnectionValue, [1], Tasks.CreateCancellationSource());\n  Net.ConnectWithCancellation('unused.invalid', 1, 1000, Tasks.CreateCancellationSource());\n  Net.ConnectTlsWithCancellation('unused.invalid', 1, 1000, Tasks.CreateCancellationSource());\nend procedure;\nbegin\nend.",
    );
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.message.contains("Type mismatch"))
            .count(),
        4,
        "errors: {errors:#?}"
    );
}

#[test]
fn go_accepts_procedure_calls_as_tasks() {
    check_ok(
        "program T;\nuses Std.Tasks as Tasks;\n\nprocedure LogAnswer();\nbegin\nend procedure;\n\nbegin scope\n  const Tsk: task := go LogAnswer();\n  Tasks.Wait(Tsk);\n end scope; end.",
    );
}

#[test]
fn go_requires_a_call_expression() {
    let (_, errors) = parse(
        "\
program T;
begin scope
  const Tsk: task := go 1;
 end scope; end.",
    );

    assert!(
        errors.iter().any(|error| match error {
            ParseDiagnostic::Parser(diagnostic) => diagnostic
                .message
                .contains("`go` requires a function or procedure call"),
            ParseDiagnostic::Lexer(_) => false,
        }),
        "errors: {errors:#?}"
    );
}

#[test]
fn task_wait_uses_task_result_type() {
    check_ok(
        "program T;\nuses Std.Tasks as Tasks;\n\nfunction Answer(): integer;\nbegin\n  return 42;\nend function;\n\nbegin scope\n  const Tsk: task of integer := go Answer();\n  const Value: integer := Tasks.Wait(Tsk);\n end scope; end.",
    );
}

#[test]
fn task_wait_reports_assignment_mismatch() {
    let errors = check_errors(
        "program T;\nuses Std.Tasks as Tasks;\n\nfunction Answer(): integer;\nbegin\n  return 42;\nend function;\n\nbegin scope\n  const Tsk: task of integer := go Answer();\n  const Value: string := Tasks.Wait(Tsk);\n end scope; end.",
    );

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Type mismatch in const initializer")),
        "errors: {errors:#?}"
    );
}

#[test]
fn go_rejects_task_bound_mutable_closure() {
    let errors = check_errors(
        "program T;\nuses Std.Tasks as Tasks;\nbegin scope\n  var Count: integer := 0;\n  const Inc: procedure() :=\n    procedure()\n    begin\n      Count := Count + 1;\n    end procedure;\n  discard go Inc();\n end scope; end.",
    );

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("task-bound")
                || error.message.contains("Task-bound")),
        "errors: {errors:#?}"
    );
}

#[test]
fn typed_channel_operations_preserve_the_element_type() {
    check_ok(
        "program T;\nuses Std.Tasks as Tasks;\nbegin\n  const Messages: channel of integer := Tasks.CreateChannel(1);\n  const Sent: result of (boolean, Tasks.TaskError) := Tasks.Send(Messages, 42);\n  const Received: result of (integer, Tasks.TaskError) := Tasks.Receive(Messages);\n  discard Tasks.CloseChannel(Messages);\nend.",
    );
}

#[test]
fn channel_send_rejects_the_wrong_element_type() {
    let errors = check_errors(
        "program T;\nuses Std.Tasks as Tasks;\nbegin\n  const Messages: channel of integer := Tasks.CreateChannel(1);\n  Tasks.Send(Messages, 'wrong');\nend.",
    );

    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("Type mismatch in channel send")),
        "errors: {errors:#?}"
    );
}

#[test]
fn channel_wait_modes_preserve_element_and_timeout_types() {
    let errors = check_errors(
        "program T;\nuses Std.Tasks as Tasks;\nbegin\n  const Messages: channel of integer := Tasks.CreateChannel(1);\n  const Pending: result of (option of integer, Tasks.TaskError) := Tasks.TryReceive(Messages);\n  Tasks.TrySend(Messages, 'wrong');\n  Tasks.SendWithTimeout(Messages, 1, 'soon');\n  Tasks.ReceiveWithTimeout(Messages, 'soon');\nend.",
    );

    assert_eq!(
        errors
            .iter()
            .filter(|error| error.message.contains("Type mismatch"))
            .count(),
        3,
        "errors: {errors:#?}"
    );
}

#[test]
fn channel_send_rejects_task_bound_values() {
    let errors = check_errors(
        "program T;\nuses Std.Tasks as Tasks;\nbegin\n  var Count: integer := 0;\n  const Work: procedure() := procedure() begin Count := Count + 1; end procedure;\n  const Queue: channel of procedure() := Tasks.CreateChannel(1);\n  Tasks.Send(Queue, Work);\nend.",
    );

    assert!(
        errors
            .iter()
            .any(|error| error.code == SEMA_TASK_BOUND_CALLABLE),
        "errors: {errors:#?}"
    );
}

#[test]
fn channel_send_rejects_task_bound_values_wrapped_in_aggregates() {
    let errors = check_errors(
        "program T;\nuses Std.Tasks as Tasks;\ntype WorkBox = record\n  Work: procedure();\nend record;\nbegin\n  var Count: integer := 0;\n  const Work: procedure() := procedure() begin Count := Count + 1; end procedure;\n  const ArrayQueue: channel of array of procedure() := Tasks.CreateChannel(1);\n  const RecordQueue: channel of WorkBox := Tasks.CreateChannel(1);\n  const ResultQueue: channel of result of (procedure(), string) := Tasks.CreateChannel(1);\n  const OptionQueue: channel of option of procedure() := Tasks.CreateChannel(1);\n  Tasks.Send(ArrayQueue, [Work]);\n  Tasks.Send(RecordQueue, WorkBox( Work := Work ));\n  Tasks.Send(ResultQueue, Ok(Work));\n  Tasks.Send(OptionQueue, Some(Work));\nend.",
    );

    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == SEMA_TASK_BOUND_CALLABLE)
            .count(),
        4,
        "errors: {errors:#?}"
    );
}

#[test]
fn channel_send_tracks_task_bound_postfix_results_by_selected_type() {
    let errors = check_errors(
        "program T;\nuses Std.Tasks as Tasks;\ntype WorkBox = record\n  Work: procedure();\n  Safe: integer;\nend record;\nbegin\n  var Count: integer := 0;\n  const Work: procedure() := procedure() begin Count := Count + 1; end procedure;\n  const Boxed: WorkBox := WorkBox( Work := Work, Safe := 7 );\n  const WorkQueue: channel of procedure() := Tasks.CreateChannel(1);\n  const SafeQueue: channel of integer := Tasks.CreateChannel(1);\n  Tasks.Send(WorkQueue, Boxed.Work);\n  discard Tasks.Send(SafeQueue, Boxed.Safe);\nend.",
    );

    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == SEMA_TASK_BOUND_CALLABLE)
            .count(),
        1,
        "errors: {errors:#?}"
    );
}

#[test]
fn typed_task_parameters_wait_for_their_declared_result_type() {
    check_ok(
        r#"program T;
uses Std.Tasks as Tasks;
function Seven(): integer;
begin
  return 7;
end function;
function Doubled(Job: task of integer): integer;
begin
  return Tasks.Wait(Job) * 2;
end function;
function First(Jobs: array of task of integer): integer;
begin
  return Tasks.Wait(Jobs[Tasks.WaitAny(Jobs)]);
end function;
begin scope
  const Job: task of integer := go Seven();
  const Typed: task of integer := go Seven();
  const Total: integer := Doubled(Job) + First([Typed]);
 end scope; end."#,
    );
}

#[test]
fn typed_tasks_reject_a_different_result_type() {
    for source in [
        "program T; uses Std.Tasks as Tasks; function Seven(): integer; begin return 7; end function; begin scope  const Job: task of string := go Seven();  end scope; end.",
        "program T; uses Std.Tasks as Tasks; function Seven(): integer; begin return 7; end function; function Name(Job: task of string): string; begin return Tasks.Wait(Job); end function; begin scope  const Job: task of integer := go Seven(); const Text: string := Name(Job);  end scope; end.",
        "program T; uses Std.Tasks as Tasks; function Count(Job: task of integer): string; begin return Tasks.Wait(Job); end function; begin end.",
    ] {
        assert!(!check_errors(source).is_empty(), "{source}");
    }
}
