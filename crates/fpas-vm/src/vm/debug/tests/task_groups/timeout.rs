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
uses Std.Tasks;
begin
  const Group: TaskGroup := CreateTaskGroup();
  const Ready: channel of boolean := CreateChannel(1);
  const Release: channel of boolean := CreateChannel(1);
  const Child: task := StartTaskInGroup(Group, function(Token: CancellationToken): integer
  begin
    discard Send(Ready, true);
    while TryReceive(Release).Unwrap().IsNone() do begin null; end; end while;
    if not IsCancellationRequested(Token) then panic('cancellation was not retained'); end if;
    return 42;
  end function);
  while TryReceive(Ready).Unwrap().IsNone() do begin null; end; end while;
  if not CloseTaskGroupWithTimeout(Group, 2).IsError() then panic('running worker was lost'); end if;
  discard Send(Release, true);
  if Wait(Child) <> 42 then panic('worker could not finish after timeout'); end if;
  if CloseTaskGroup(Group).Length() <> 0 then panic('close failed'); end if;
  discard CloseChannel(Ready); discard CloseChannel(Release);
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
uses Std.Tasks;
begin
  const Outer: TaskGroup := CreateTaskGroup();
  const Ready: channel of boolean := CreateChannel(1);
  const Gate: channel of integer := CreateChannel(1);
  const Parent: task := StartTaskInGroup(Outer, procedure(Token: CancellationToken)
  begin
    const Inner: TaskGroup := CreateTaskGroup();
    const Child: task := StartTaskInGroup(Inner, function(Stop: CancellationToken): integer
      begin return Receive(Gate).Unwrap(); end function);
    if not CloseTaskGroupWithTimeout(Inner, 2).IsError() then panic('premature close'); end if;
    discard Send(Ready, true).Unwrap();
    if Wait(Child) <> 42 then panic('child result was lost'); end if;
    if CloseTaskGroupWithTimeout(Inner, 1000).Unwrap().Length() <> 0 then panic('inner failures'); end if;
  end procedure);
  discard Receive(Ready).Unwrap();
  discard Send(Gate, 42).Unwrap();
  if CloseTaskGroupWithTimeout(Outer, 1000).Unwrap().Length() <> 0 then panic('outer failures'); end if;
  discard CloseChannel(Ready); discard CloseChannel(Gate);
end."#,
    );
}
