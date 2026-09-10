//! Timed group close retains ownership in normal, single-worker, and debugger execution.

use super::*;

fn run_normal(executable: fpas_bytecode::VerifiedExecutable, pool_size: Option<usize>) {
    let mut vm = crate::vm::Vm::new(executable);
    if let Some(size) = pool_size {
        vm.pool_size = size;
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
        result.expect("normal timed close");
    });
}

fn run_modes(source: &str) {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let executable = fpas_compiler::compile(&program).expect("compile");
    for pool_size in [None, Some(1)] {
        run_normal(executable.clone(), pool_size);
    }
    let mut session = DebugSession::with_manual_clock(executable).expect("debugger");
    assert!(matches!(
        session.continue_execution().expect("debug timed close"),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn timed_group_close_returns_while_a_running_pool_worker_ignores_cancellation() {
    let (program, errors) = fpas_parser::parse(
        r#"program NonCooperativeWorker;
uses Std.Task, Std.Results, Std.Options, Std.Arrays;
begin
  var Group: TaskGroup := CreateTaskGroup();
  var Ready: channel of boolean := CreateChannel(1);
  var Release: channel of boolean := CreateChannel(1);
  var Child: task := StartTaskInGroup(Group, function(Token: CancellationToken): integer
  begin
    Send(Ready, true);
    while IsNone(Std.Results.Unwrap(TryReceive(Release))) do begin end;
    if not IsCancellationRequested(Token) then panic('cancellation was not retained');
    return 42
  end);
  while IsNone(Std.Results.Unwrap(TryReceive(Ready))) do begin end;
  if not IsError(CloseTaskGroupWithTimeout(Group, 2)) then panic('running worker was lost');
  Send(Release, true);
  if Wait(Child) <> 42 then panic('worker could not finish after timeout');
  if Length(CloseTaskGroup(Group)) <> 0 then panic('close failed');
  CloseChannel(Ready); CloseChannel(Release)
end."#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let executable = fpas_compiler::compile(&program).expect("compile");
    for pool_size in [None, Some(1)] {
        run_normal(executable.clone(), pool_size);
    }
}

#[test]
fn timed_group_close_retains_blocked_worker_until_a_later_successful_close() {
    run_modes(include_str!(
        "../../../../../../../tests/concurrency/task_group_close_timeout_test.fpas"
    ));
}

#[test]
fn child_timed_group_close_yields_to_its_waiting_parent() {
    run_modes(
        r#"program NestedClose;
uses Std.Task, Std.Results, Std.Arrays;
begin
  var Outer: TaskGroup := CreateTaskGroup();
  var Ready: channel of boolean := CreateChannel(1);
  var Gate: channel of integer := CreateChannel(1);
  var Parent: task := StartTaskInGroup(Outer, procedure(Token: CancellationToken)
  begin
    var Inner: TaskGroup := CreateTaskGroup();
    var Child: task := StartTaskInGroup(Inner, function(Stop: CancellationToken): integer
      begin return Unwrap(Receive(Gate)) end);
    if not IsError(CloseTaskGroupWithTimeout(Inner, 2)) then panic('premature close');
    Unwrap(Send(Ready, true));
    if Wait(Child) <> 42 then panic('child result was lost');
    if Length(Unwrap(CloseTaskGroupWithTimeout(Inner, 1000))) <> 0 then panic('inner failures')
  end);
  Unwrap(Receive(Ready));
  Unwrap(Send(Gate, 42));
  if Length(Unwrap(CloseTaskGroupWithTimeout(Outer, 1000))) <> 0 then panic('outer failures');
  CloseChannel(Ready); CloseChannel(Gate)
end."#,
    );
}
