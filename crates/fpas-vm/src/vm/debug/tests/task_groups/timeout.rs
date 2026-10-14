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
uses Std.Tasks as Tasks;
begin
  const Group: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  const Ready: channel of boolean := Tasks.CreateChannel(1);
  const Release: channel of boolean := Tasks.CreateChannel(1);
  const Child: task of integer := Tasks.StartTaskInGroup(Group, function(Token: Tasks.CancellationToken): integer
  begin
    discard Tasks.Send(Ready, true);
    while (Tasks.TryReceive(Release).Unwrap() is None) do begin null; end; end while;
    if not Tasks.IsCancellationRequested(Token) then panic('cancellation was not retained'); end if;
    return 42;
  end function);
  while (Tasks.TryReceive(Ready).Unwrap() is None) do begin null; end; end while;
  if not (Tasks.CloseTaskGroupWithTimeout(Group, 2) is Error(_)) then panic('running worker was lost'); end if;
  discard Tasks.Send(Release, true);
  if Tasks.Wait(Child) <> 42 then panic('worker could not finish after timeout'); end if;
  if Tasks.CloseTaskGroup(Group).Length() <> 0 then panic('close failed'); end if;
  discard Tasks.CloseChannel(Ready); discard Tasks.CloseChannel(Release);
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
uses Std.Tasks as Tasks;
begin
  const Outer: Tasks.TaskGroup := Tasks.CreateTaskGroup();
  const Ready: channel of boolean := Tasks.CreateChannel(1);
  const Gate: channel of integer := Tasks.CreateChannel(1);
  const Parent: task := Tasks.StartTaskInGroup(Outer, procedure(Token: Tasks.CancellationToken)
  begin
    const Inner: Tasks.TaskGroup := Tasks.CreateTaskGroup();
    const Child: task of integer := Tasks.StartTaskInGroup(Inner, function(Stop: Tasks.CancellationToken): integer
      begin return Tasks.Receive(Gate).Unwrap(); end function);
    if not (Tasks.CloseTaskGroupWithTimeout(Inner, 2) is Error(_)) then panic('premature close'); end if;
    discard Tasks.Send(Ready, true).Unwrap();
    if Tasks.Wait(Child) <> 42 then panic('child result was lost'); end if;
    if Tasks.CloseTaskGroupWithTimeout(Inner, 1000).Unwrap().Length() <> 0 then panic('inner failures'); end if;
  end procedure);
  discard Tasks.Receive(Ready).Unwrap();
  discard Tasks.Send(Gate, 42).Unwrap();
  if Tasks.CloseTaskGroupWithTimeout(Outer, 1000).Unwrap().Length() <> 0 then panic('outer failures'); end if;
  discard Tasks.CloseChannel(Ready); discard Tasks.CloseChannel(Gate);
end."#,
    );
}
