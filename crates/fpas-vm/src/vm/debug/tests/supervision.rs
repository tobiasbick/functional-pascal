//! Supervised attempts retain their identity, inputs, and final outcome across schedulers.

use super::*;

mod channel_compatibility;
mod entry_completion;

fn run_both(source: &str) {
    run_with_children(source, 1);
}

fn run_with_children(source: &str, expected_children: usize) {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let executable = fpas_compiler::compile(&program).expect("compile");
    for pool_size in [None, Some(1)] {
        let mut vm = crate::vm::Vm::new(executable.clone());
        assert!(
            vm.pool_size > 0,
            "task-start intrinsic did not activate the pool"
        );
        if let Some(pool_size) = pool_size {
            vm.pool_size = pool_size;
        }
        let shutdown = vm.shutdown_handle();
        let (done, finished) = std::sync::mpsc::channel();
        std::thread::scope(|scope| {
            let watchdog = scope.spawn(move || {
                if finished
                    .recv_timeout(std::time::Duration::from_secs(30))
                    .is_err()
                {
                    shutdown.shutdown();
                }
            });
            let result = vm.run();
            let _ = done.send(());
            watchdog.join().expect("watchdog");
            assert!(result.is_ok(), "pool {pool_size:?}: {result:?}");
        });
    }
    let mut session = DebugSession::with_manual_clock(executable).expect("session");
    let result = session.continue_execution().expect("debug supervisor");
    assert!(
        matches!(result, DebugRunResult::Terminated(_)),
        "{result:?}"
    );
    let events = session.take_task_events();
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == crate::DebugTaskEventKind::Started)
            .count(),
        expected_children,
        "retry created a new task identity"
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == crate::DebugTaskEventKind::Exited)
            .count(),
        expected_children,
        "transient failure emitted an exit event"
    );
}

#[test]
fn supervision_retries_a_panic_observed_when_a_parked_selection_resumes() {
    run_with_children(
        r#"program ResumeFailure;
uses Std.Task, Std.Array, Std.Result, Std.Time;
begin
  var Group: TaskGroup := CreateTaskGroup();
  var Attempts: channel of boolean := CreateChannel(2);
  Send(Attempts, false); Send(Attempts, true);
  var Parent: task := StartSupervisedTask(Group, function(Token: CancellationToken): result of integer, string
  begin
    if Unwrap(Receive(Attempts)) then return Ok(42);
    var Child: task := StartTaskInGroup(Group, procedure(ChildToken: CancellationToken)
      begin Sleep(2); panic('owned child panic') end);
    Select([TaskCase(Child, procedure() begin panic('failed task selected') end)]);
    return Error('unreachable')
  end, 1, 1);
  if Unwrap(Wait(Parent)) <> 42 then panic('resume failure was not retried');
  var Failures: array of TaskFailure := CloseTaskGroup(Group);
  if Length(Failures) <> 1 then panic('wrong group failure count');
  if Failures[0].Kind <> TaskFailureKind.Panicked then panic('child failure lost');
  CloseChannel(Attempts)
end."#,
        2,
    );
}

#[test]
fn selection_producer_yields_to_its_waiting_consumer() {
    run_both(
        r#"program ProducerConsumer;
uses Std.Task, Std.Result, Std.Array;
begin
  var Group: TaskGroup := CreateTaskGroup();
  var Queue: channel of integer := CreateChannel(1);
  var Child: task := StartTaskInGroup(Group, procedure(Token: CancellationToken)
  begin
    for Item: integer := 1 to 2 do
      Select([SendCase(Queue, Item, procedure(Outcome: result of boolean, string) begin end)])
  end);
  mutable var Total: integer := 0;
  for Item: integer := 1 to 2 do
    Select([ReceiveCase(Queue, procedure(Outcome: result of integer, string)
      begin Total := Total + Unwrap(Outcome) end)]);
  Wait(Child);
  if Total <> 3 then panic('delivery lost');
  if Length(CloseTaskGroup(Group)) <> 0 then panic('worker failed');
  CloseChannel(Queue)
end."#,
    );
}

#[test]
fn supervision_and_channel_selection_drain_contending_workers_across_repeated_groups() {
    run_with_children(
        include_str!("../../../../../../tests/concurrency/supervised_channel_selection_test.fpas"),
        32,
    );
}

#[test]
fn supervision_retries_keep_nested_children_in_the_original_group() {
    run_with_children(
        r#"program NestedAttempts;
uses Std.Task, Std.Array, Std.Result, Std.Time;
begin
  var G: TaskGroup := CreateTaskGroup();
  var Steps: channel of boolean := CreateChannel(2);
  Send(Steps, false); Send(Steps, true);
  var Parent: task := StartSupervisedTask(G, function(Token: CancellationToken): result of integer, string
  begin
    var Child: task := StartTaskInGroup(G, function(ChildToken: CancellationToken): integer
    begin Sleep(1); return 42 end);
    var Answer: integer := Wait(Child);
    if not Unwrap(Receive(Steps)) then return Error('retry parent');
    return Ok(Answer)
  end, 1, 1);
  Select([TaskCase(Parent, procedure() begin end)]);
  if Unwrap(Wait(Parent)) <> 42 then panic('nested result lost');
  if Length(CloseTaskGroup(G)) <> 0 then panic('transient attempt reported failure');
  CloseChannel(Steps)
end."#,
        3,
    );
}

#[test]
fn supervision_successful_procedure_does_not_use_retry_budget() {
    run_both(
        r#"program SuccessfulProcedure;
uses Std.Task, Std.Array, Std.Result;
begin
  var G: TaskGroup := CreateTaskGroup();
  var Attempts: channel of integer := CreateChannel(2);
  var Child: task := StartSupervisedTask(G, procedure(Token: CancellationToken)
  begin Send(Attempts, 1) end, 1023, 60000);
  Wait(Child);
  if Unwrap(Receive(Attempts)) <> 1 then panic('missing attempt');
  case Unwrap(TryReceive(Attempts)) of Some(_): panic('success retried'); None: begin end end;
  if Length(CloseTaskGroup(G)) <> 0 then panic('successful procedure reported failure');
  CloseChannel(Attempts)
end."#,
    );
}

#[test]
fn supervision_successful_value_is_terminal_even_after_worker_requests_cancellation() {
    run_both(
        r#"program SuccessfulCancellation;
uses Std.Task, Std.Array;
begin
  var G: TaskGroup := CreateTaskGroup();
  var Child: task := StartSupervisedTask(G, function(Token: CancellationToken): integer
  begin CancelTaskGroup(G); return 42 end, 1023, 60000);
  if Wait(Child) <> 42 then panic('successful value lost');
  if Length(CloseTaskGroup(G)) <> 0 then panic('success replaced with cancellation')
end."#,
    );
}

#[test]
fn supervision_retries_errors_and_panics_then_delivers_one_successful_task() {
    run_both(
        r#"program RecoverWorker;
uses Std.Task, Std.Array, Std.Result;
begin
  var G: TaskGroup := CreateTaskGroup();
  var Steps: channel of integer := CreateChannel(3);
  Send(Steps, 0); Send(Steps, 1); Send(Steps, 2);
  var Original: array of integer := [0];
  var Child: task := StartSupervisedTask(G, function(Token: CancellationToken): result of integer, string
  begin
    mutable var Local: array of integer := Original;
    Local[0] := Local[0] + 1;
    if Local[0] <> 1 then panic('attempt capture leaked');
    var Step: integer := Unwrap(Receive(Steps));
    if Step = 0 then return Error('retryable');
    if Step = 1 then panic('retryable panic');
    return Ok(42)
  end, 2, 1);
  mutable var Seen: boolean := false;
  var Ready: WaitCase := TaskCase(Child, procedure() begin Seen := true end);
  if Select([Ready]) <> 0 then panic('completion index');
  if not Seen then panic('missing completion');
  if Unwrap(Wait(Child)) <> 42 then panic('result lost');
  if Original[0] <> 0 then panic('original capture changed');
  if Length(CloseTaskGroup(G)) <> 0 then panic('transient failures escaped');
  CloseChannel(Steps)
end."#,
    );
}

#[test]
fn supervision_error_exhaustion_keeps_the_final_result_and_one_group_report() {
    run_both(
        r#"program ExhaustWorker;
uses Std.Task, Std.Array, Std.Result;
begin
  var G: TaskGroup := CreateTaskGroup();
  var Attempts: channel of integer := CreateChannel(3);
  var Child: task := StartSupervisedTask(G, function(Token: CancellationToken): result of integer, string
  begin Send(Attempts, 1); return Error('last failure') end, 2, 0);
  case Wait(Child) of
    Ok(_): panic('unexpected success');
    Error(Message): if Message <> 'last failure' then panic('wrong final error')
  end;
  var Failures: array of TaskFailure := CloseTaskGroup(G);
  if Length(Failures) <> 1 then panic('attempts became children');
  if Failures[0].Kind <> TaskFailureKind.ReturnedError then panic('wrong failure kind');
  for I: integer := 1 to 3 do if Unwrap(Receive(Attempts)) <> 1 then panic('attempt count');
  CloseChannel(Attempts)
end."#,
    );
}

#[test]
fn supervision_panic_exhaustion_is_contained_and_keeps_the_last_diagnostic() {
    run_both(
        r#"program ExhaustPanic;
uses Std.Task, Std.Array, Std.Result;
begin
  var G: TaskGroup := CreateTaskGroup();
  var Attempts: channel of integer := CreateChannel(1);
  StartSupervisedTask(G, procedure(Token: CancellationToken)
  begin Send(Attempts, 1); panic('last panic') end, 2, 1);
  for I: integer := 1 to 3 do Unwrap(Receive(Attempts));
  var Failures: array of TaskFailure := CloseTaskGroup(G);
  if Length(Failures) <> 1 then panic('failure count');
  if Failures[0].Kind <> TaskFailureKind.Panicked then panic('panic category');
  if Failures[0].Message <> 'panic: last panic' then panic('panic message');
  CloseChannel(Attempts)
end."#,
    );
}

#[test]
fn supervision_cancel_interrupts_a_long_backoff() {
    run_both(
        r#"program CancelBackoff;
uses Std.Task, Std.Array, Std.Result;
begin
  var G: TaskGroup := CreateTaskGroup();
  var Attempts: channel of boolean := CreateChannel(1);
  StartSupervisedTask(G, function(Token: CancellationToken): result of integer, string
  begin Send(Attempts, true); return Error('retry later') end, 3, 60000);
  Unwrap(Receive(Attempts));
  Select([TimerCase(2, procedure() begin end)]);
  CancelTaskGroup(G);
  var Failures: array of TaskFailure := CloseTaskGroup(G);
  if Length(Failures) <> 1 then panic('failure count');
  if Failures[0].Kind <> TaskFailureKind.Cancelled then panic('not cancelled');
  CloseChannel(Attempts)
end."#,
    );
}

#[test]
fn supervision_does_not_retry_other_runtime_errors() {
    run_both(
        r#"program InvalidWorkerOperation;
uses Std.Task, Std.Array, Std.Result;
begin
  var G: TaskGroup := CreateTaskGroup();
  var Started: channel of boolean := CreateChannel(1);
  StartSupervisedTask(G, procedure(Token: CancellationToken)
  begin Send(Started, true); CloseTaskGroup(G) end, 3, 60000);
  Unwrap(Receive(Started));
  var Failures: array of TaskFailure := CloseTaskGroup(G);
  if Length(Failures) <> 1 then panic('failure count');
  if Failures[0].Kind <> TaskFailureKind.RuntimeError then panic('runtime error was retried');
  CloseChannel(Started)
end."#,
    );
}

#[test]
fn supervision_zero_retry_limit_keeps_an_ordinary_cancelled_message_as_error() {
    run_both(
        r#"program NoRetries;
uses Std.Task, Std.Array;
begin
  var G: TaskGroup := CreateTaskGroup();
  var Child: task := StartSupervisedTask(G, function(Token: CancellationToken): result of integer, string
  begin return Error('cancelled') end, 0, 60000);
  case Wait(Child) of Ok(_): panic('unexpected success'); Error(_): begin end end;
  var Failures: array of TaskFailure := CloseTaskGroup(G);
  if Length(Failures) <> 1 then panic('failure count');
  if Failures[0].Kind <> TaskFailureKind.ReturnedError then panic('message guessed cancellation')
end."#,
    );
}
