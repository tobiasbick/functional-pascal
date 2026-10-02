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
uses Std.Tasks as Tasks; uses Std.Arrays as Arrays; uses Std.Results as Results; uses Std.Time as Time;
begin
  var Group: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Attempts: channel of boolean := Tasks.CreateChannel(2);
  Tasks.Send(Attempts, false); Tasks.Send(Attempts, true);
  var Parent: task := Tasks.StartSupervisedTask(Group, function(Token: Tasks.CancellationToken): result of integer, string
  begin
    if Results.Unwrap(Tasks.Receive(Attempts)) then return Ok(42); end if;
    var Child: task := Tasks.StartTaskInGroup(Group, procedure(ChildToken: Tasks.CancellationToken)
      begin Time.Sleep(2); panic('owned child panic'); end procedure);
    Tasks.Select([Tasks.TaskCase(Child, procedure() begin panic('failed task selected'); end procedure)]);
    return Error('unreachable');
  end function, 1, 1);
  if Results.Unwrap(Tasks.Wait(Parent)) <> 42 then panic('resume failure was not retried'); end if;
  var Failures: array of Tasks.TaskFailure := Tasks.CloseTaskGroup(Group);
  if Arrays.Length(Failures) <> 1 then panic('wrong group failure count'); end if;
  if Failures[0].Kind <> Tasks.TaskFailureKind.Panicked then panic('child failure lost'); end if;
  Tasks.CloseChannel(Attempts);
end program;"#,
        2,
    );
}

#[test]
fn selection_producer_yields_to_its_waiting_consumer() {
    run_both(
        r#"program ProducerConsumer;
uses Std.Tasks as Tasks; uses Std.Results as Results; uses Std.Arrays as Arrays;
begin
  var Group: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Queue: channel of integer := Tasks.CreateChannel(1);
  var Child: task := Tasks.StartTaskInGroup(Group, procedure(Token: Tasks.CancellationToken)
  begin
    for Item: integer := 1 to 2 do
      Tasks.Select([Tasks.SendCase(Queue, Item, procedure(Outcome: result of boolean, string) begin null; end procedure)]); end for;
  end procedure);
  mutable var Total: integer := 0;
  for Item: integer := 1 to 2 do
    Tasks.Select([Tasks.ReceiveCase(Queue, procedure(Outcome: result of integer, string)
      begin Total := Total + Results.Unwrap(Outcome); end procedure)]); end for;
  Tasks.Wait(Child);
  if Total <> 3 then panic('delivery lost'); end if;
  if Arrays.Length(Tasks.CloseTaskGroup(Group)) <> 0 then panic('worker failed'); end if;
  Tasks.CloseChannel(Queue);
end program;"#,
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
uses Std.Tasks as Tasks; uses Std.Arrays as Arrays; uses Std.Results as Results; uses Std.Time as Time;
begin
  var G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Steps: channel of boolean := Tasks.CreateChannel(2);
  Tasks.Send(Steps, false); Tasks.Send(Steps, true);
  var Parent: task := Tasks.StartSupervisedTask(G, function(Token: Tasks.CancellationToken): result of integer, string
  begin
    var Child: task := Tasks.StartTaskInGroup(G, function(ChildToken: Tasks.CancellationToken): integer
    begin Time.Sleep(1); return 42; end function);
    var Answer: integer := Tasks.Wait(Child);
    if not Results.Unwrap(Tasks.Receive(Steps)) then return Error('retry parent'); end if;
    return Ok(Answer);
  end function, 1, 1);
  Tasks.Select([Tasks.TaskCase(Parent, procedure() begin null; end procedure)]);
  if Results.Unwrap(Tasks.Wait(Parent)) <> 42 then panic('nested result lost'); end if;
  if Arrays.Length(Tasks.CloseTaskGroup(G)) <> 0 then panic('transient attempt reported failure'); end if;
  Tasks.CloseChannel(Steps);
end program;"#,
        3,
    );
}

#[test]
fn supervision_successful_procedure_does_not_use_retry_budget() {
    run_both(
        r#"program SuccessfulProcedure;
uses Std.Tasks as Tasks; uses Std.Arrays as Arrays; uses Std.Results as Results;
begin
  var G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Attempts: channel of integer := Tasks.CreateChannel(2);
  var Child: task := Tasks.StartSupervisedTask(G, procedure(Token: Tasks.CancellationToken)
  begin Tasks.Send(Attempts, 1); end procedure, 1023, 60000);
  Tasks.Wait(Child);
  if Results.Unwrap(Tasks.Receive(Attempts)) <> 1 then panic('missing attempt'); end if;
  case Results.Unwrap(Tasks.TryReceive(Attempts)) of when Some(_): panic('success retried'); when None: begin null; end; end case;
  if Arrays.Length(Tasks.CloseTaskGroup(G)) <> 0 then panic('successful procedure reported failure'); end if;
  Tasks.CloseChannel(Attempts);
end program;"#,
    );
}

#[test]
fn supervision_successful_value_is_terminal_even_after_worker_requests_cancellation() {
    run_both(
        r#"program SuccessfulCancellation;
uses Std.Tasks as Tasks; uses Std.Arrays as Arrays;
begin
  var G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Child: task := Tasks.StartSupervisedTask(G, function(Token: Tasks.CancellationToken): integer
  begin Tasks.CancelTaskGroup(G); return 42; end function, 1023, 60000);
  if Tasks.Wait(Child) <> 42 then panic('successful value lost'); end if;
  if Arrays.Length(Tasks.CloseTaskGroup(G)) <> 0 then panic('success replaced with cancellation'); end if;
end program;"#,
    );
}

#[test]
fn supervision_retries_errors_and_panics_then_delivers_one_successful_task() {
    run_both(
        r#"program RecoverWorker;
uses Std.Tasks as Tasks; uses Std.Arrays as Arrays; uses Std.Results as Results;
begin
  var G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Steps: channel of integer := Tasks.CreateChannel(3);
  Tasks.Send(Steps, 0); Tasks.Send(Steps, 1); Tasks.Send(Steps, 2);
  var Original: array of integer := [0];
  var Child: task := Tasks.StartSupervisedTask(G, function(Token: Tasks.CancellationToken): result of integer, string
  begin
    mutable var Local: array of integer := Original;
    Local[0] := Local[0] + 1;
    if Local[0] <> 1 then panic('attempt capture leaked'); end if;
    var Step: integer := Results.Unwrap(Tasks.Receive(Steps));
    if Step = 0 then return Error('retryable'); end if;
    if Step = 1 then panic('retryable panic'); end if;
    return Ok(42);
  end function, 2, 1);
  mutable var Seen: boolean := false;
  var Ready: Tasks.WaitCase := Tasks.TaskCase(Child, procedure() begin Seen := true; end procedure);
  if Tasks.Select([Ready]) <> 0 then panic('completion index'); end if;
  if not Seen then panic('missing completion'); end if;
  if Results.Unwrap(Tasks.Wait(Child)) <> 42 then panic('result lost'); end if;
  if Original[0] <> 0 then panic('original capture changed'); end if;
  if Arrays.Length(Tasks.CloseTaskGroup(G)) <> 0 then panic('transient failures escaped'); end if;
  Tasks.CloseChannel(Steps);
end program;"#,
    );
}

#[test]
fn supervision_error_exhaustion_keeps_the_final_result_and_one_group_report() {
    run_both(
        r#"program ExhaustWorker;
uses Std.Tasks as Tasks; uses Std.Arrays as Arrays; uses Std.Results as Results;
begin
  var G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Attempts: channel of integer := Tasks.CreateChannel(3);
  var Child: task := Tasks.StartSupervisedTask(G, function(Token: Tasks.CancellationToken): result of integer, string
  begin Tasks.Send(Attempts, 1); return Error('last failure'); end function, 2, 0);
  case Tasks.Wait(Child) of
    when Ok(_): panic('unexpected success');
    when Error(Message): if Message <> 'last failure' then panic('wrong final error'); end if;
  end case;
  var Failures: array of Tasks.TaskFailure := Tasks.CloseTaskGroup(G);
  if Arrays.Length(Failures) <> 1 then panic('attempts became children'); end if;
  if Failures[0].Kind <> Tasks.TaskFailureKind.ReturnedError then panic('wrong failure kind'); end if;
  for I: integer := 1 to 3 do if Results.Unwrap(Tasks.Receive(Attempts)) <> 1 then panic('attempt count'); end if; end for;
  Tasks.CloseChannel(Attempts);
end program;"#,
    );
}

#[test]
fn supervision_panic_exhaustion_is_contained_and_keeps_the_last_diagnostic() {
    run_both(
        r#"program ExhaustPanic;
uses Std.Tasks as Tasks; uses Std.Arrays as Arrays; uses Std.Results as Results;
begin
  var G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Attempts: channel of integer := Tasks.CreateChannel(1);
  Tasks.StartSupervisedTask(G, procedure(Token: Tasks.CancellationToken)
  begin Tasks.Send(Attempts, 1); panic('last panic'); end procedure, 2, 1);
  for I: integer := 1 to 3 do Results.Unwrap(Tasks.Receive(Attempts)); end for;
  var Failures: array of Tasks.TaskFailure := Tasks.CloseTaskGroup(G);
  if Arrays.Length(Failures) <> 1 then panic('failure count'); end if;
  if Failures[0].Kind <> Tasks.TaskFailureKind.Panicked then panic('panic category'); end if;
  if Failures[0].Message <> 'panic: last panic' then panic('panic message'); end if;
  Tasks.CloseChannel(Attempts);
end program;"#,
    );
}

#[test]
fn supervision_cancel_interrupts_a_long_backoff() {
    run_both(
        r#"program CancelBackoff;
uses Std.Tasks as Tasks; uses Std.Arrays as Arrays; uses Std.Results as Results;
begin
  var G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Attempts: channel of boolean := Tasks.CreateChannel(1);
  Tasks.StartSupervisedTask(G, function(Token: Tasks.CancellationToken): result of integer, string
  begin Tasks.Send(Attempts, true); return Error('retry later'); end function, 3, 60000);
  Results.Unwrap(Tasks.Receive(Attempts));
  Tasks.Select([Tasks.TimerCase(2, procedure() begin null; end procedure)]);
  Tasks.CancelTaskGroup(G);
  var Failures: array of Tasks.TaskFailure := Tasks.CloseTaskGroup(G);
  if Arrays.Length(Failures) <> 1 then panic('failure count'); end if;
  if Failures[0].Kind <> Tasks.TaskFailureKind.Cancelled then panic('not cancelled'); end if;
  Tasks.CloseChannel(Attempts);
end program;"#,
    );
}

#[test]
fn supervision_does_not_retry_other_runtime_errors() {
    run_both(
        r#"program InvalidWorkerOperation;
uses Std.Tasks as Tasks; uses Std.Arrays as Arrays; uses Std.Results as Results;
begin
  var G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Started: channel of boolean := Tasks.CreateChannel(1);
  Tasks.StartSupervisedTask(G, procedure(Token: Tasks.CancellationToken)
  begin Tasks.Send(Started, true); Tasks.CloseTaskGroup(G); end procedure, 3, 60000);
  Results.Unwrap(Tasks.Receive(Started));
  var Failures: array of Tasks.TaskFailure := Tasks.CloseTaskGroup(G);
  if Arrays.Length(Failures) <> 1 then panic('failure count'); end if;
  if Failures[0].Kind <> Tasks.TaskFailureKind.RuntimeError then panic('runtime error was retried'); end if;
  Tasks.CloseChannel(Started);
end program;"#,
    );
}

#[test]
fn supervision_zero_retry_limit_keeps_an_ordinary_cancelled_message_as_error() {
    run_both(
        r#"program NoRetries;
uses Std.Tasks as Tasks; uses Std.Arrays as Arrays;
begin
  var G: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Child: task := Tasks.StartSupervisedTask(G, function(Token: Tasks.CancellationToken): result of integer, string
  begin return Error('cancelled'); end function, 0, 60000);
  case Tasks.Wait(Child) of when Ok(_): panic('unexpected success'); when Error(_): begin null; end; end case;
  var Failures: array of Tasks.TaskFailure := Tasks.CloseTaskGroup(G);
  if Arrays.Length(Failures) <> 1 then panic('failure count'); end if;
  if Failures[0].Kind <> Tasks.TaskFailureKind.ReturnedError then panic('message guessed cancellation'); end if;
end program;"#,
    );
}
