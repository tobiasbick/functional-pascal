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
uses Std.Tasks as Tasks; uses Std.Results as Results; uses Std.Options as Options; uses Std.Arrays as Arrays;
begin
  var Group: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Ready: channel of boolean := Tasks.CreateChannel(1);
  var Release: channel of boolean := Tasks.CreateChannel(1);
  var Child: task := Tasks.StartTaskInGroup(Group, function(Token: Tasks.CancellationToken): integer
  begin
    discard Tasks.Send(Ready, true);
    while Options.IsNone(Results.Unwrap(Tasks.TryReceive(Release))) do begin null; end; end while;
    if not Tasks.IsCancellationRequested(Token) then panic('cancellation was not retained'); end if;
    return 42;
  end function);
  while Options.IsNone(Results.Unwrap(Tasks.TryReceive(Ready))) do begin null; end; end while;
  if not Results.IsError(Tasks.CloseTaskGroupWithTimeout(Group, 2)) then panic('running worker was lost'); end if;
  discard Tasks.Send(Release, true);
  if Tasks.Wait(Child) <> 42 then panic('worker could not finish after timeout'); end if;
  if Arrays.Length(Tasks.CloseTaskGroup(Group)) <> 0 then panic('close failed'); end if;
  discard Tasks.CloseChannel(Ready); discard Tasks.CloseChannel(Release);
end program;"#,
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
uses Std.Tasks as Tasks; uses Std.Results as Results; uses Std.Arrays as Arrays;
begin
  var Outer: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  var Ready: channel of boolean := Tasks.CreateChannel(1);
  var Gate: channel of integer := Tasks.CreateChannel(1);
  var Parent: task := Tasks.StartTaskInGroup(Outer, procedure(Token: Tasks.CancellationToken)
  begin
    var Inner: Tasks.TaskGroup := Tasks.CreateTaskGroup();
    var Child: task := Tasks.StartTaskInGroup(Inner, function(Stop: Tasks.CancellationToken): integer
      begin return Results.Unwrap(Tasks.Receive(Gate)); end function);
    if not Results.IsError(Tasks.CloseTaskGroupWithTimeout(Inner, 2)) then panic('premature close'); end if;
    discard Results.Unwrap(Tasks.Send(Ready, true));
    if Tasks.Wait(Child) <> 42 then panic('child result was lost'); end if;
    if Arrays.Length(Results.Unwrap(Tasks.CloseTaskGroupWithTimeout(Inner, 1000))) <> 0 then panic('inner failures'); end if;
  end procedure);
  discard Results.Unwrap(Tasks.Receive(Ready));
  discard Results.Unwrap(Tasks.Send(Gate, 42));
  if Arrays.Length(Results.Unwrap(Tasks.CloseTaskGroupWithTimeout(Outer, 1000))) <> 0 then panic('outer failures'); end if;
  discard Tasks.CloseChannel(Ready); discard Tasks.CloseChannel(Gate);
end program;"#,
    );
}
