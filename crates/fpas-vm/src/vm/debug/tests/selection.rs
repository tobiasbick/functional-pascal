//! Mixed-source selection uses the same resumable callbacks in the VM and debugger.

use super::*;

const SOURCE: &str = r#"program SelectionContinuation;

uses Std.Tasks as Tasks;
uses Std.Time as Time;
uses Std.Results as Results;

function Produce(Q: channel of (integer)): integer;
begin
  Time.Sleep(2);
  discard Tasks.Send(Q, 123);
  return 1;
end function;

function Child(): integer;
begin
  Time.Sleep(2);
  return 7;
end function;

function Parent(): integer;
begin
  const T: task := go Child();
   var Seen: integer := 0;
  const C: Tasks.WaitCase := Tasks.TaskCase(T, procedure() begin
    Time.Sleep(0);
    Time.Sleep(1);
    Seen := Tasks.Wait(T);
  end procedure);
  if Tasks.Select([C]) <> 0 then
    panic('task index');
  end if;

  return Seen;
end function;

begin
  const T: task := go Parent();
  const Q: channel of (integer) := Tasks.CreateChannel(1);
   var Seen: integer := 0;
  const S: Tasks.WaitCase := Tasks.SendCase(Q, 42, procedure(R: Result of (boolean, string)) begin
    if not Results.Unwrap(R) then
      panic('send result');
    end if;

    Time.Sleep(1);
    const Receive: Tasks.WaitCase := Tasks.ReceiveCase(Q, procedure(V: Result of (integer, string)) begin
      Seen := Results.Unwrap(V);
    end procedure);
    if Tasks.Select([Receive]) <> 0 then
      panic('nested selection');
    end if;
  end procedure);
  if Tasks.Select([S]) <> 0 then
    panic('send index');
  end if;

  if Seen <> 42 then
    panic('callback did not complete');
  end if;

  if Tasks.Wait(T) <> 7 then
    panic('task value lost');
  end if;
  const Producer: task := go Produce(Q);
  const Pending: Tasks.WaitCase := Tasks.ReceiveCase(Q, procedure(R: Result of (integer, string)) begin
    Seen := Results.Unwrap(R);
  end procedure);
  const Fallback: Tasks.WaitCase := Tasks.TimerCase(1000, procedure() begin
    panic('pending receive timed out');
  end procedure);
  if Tasks.Select([Pending, Fallback]) <> 0 then
    panic('pending receive index');
  end if;

  if Seen <> 123 then
    panic('pending receive value');
  end if;

  if Tasks.Wait(Producer) <> 1 then
    panic('producer result');
  end if;
  const Closed: Tasks.WaitCase := Tasks.ReceiveCase(Q, procedure(R: Result of (integer, string)) begin
    case R of
      when Result.Ok(_):
        panic('closed channel delivered');
      when Result.Error(const Message):
        if Message <> 'Channel is closed' then
          panic(Message);
        end if;
    end case;
  end procedure);
  discard Tasks.CloseChannel(Q);
  if Tasks.Select([Closed]) <> 0 then
    panic('closed index');
  end if;
  const Timer: Tasks.WaitCase := Tasks.TimerCase(2, procedure() begin
    Seen := 99;
  end procedure);
  if Tasks.Select([Timer]) <> 0 then
    panic('timer index');
  end if;

  if Seen <> 99 then
    panic('timer callback');
  end if;
end program;
"#;

#[test]
fn selection_callbacks_resume_nested_waits_with_one_worker() {
    let (program, errors) = fpas_parser::parse(SOURCE);
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::vm::Vm::new(fpas_compiler::compile(&program).expect("compile"));
    vm.pool_size = 1;
    vm.run().expect("one-worker selection");
}

#[test]
fn selection_callbacks_and_timers_use_deterministic_debugger_execution() {
    let (program, errors) = fpas_parser::parse(SOURCE);
    assert!(errors.is_empty(), "{errors:?}");
    let mut session =
        DebugSession::with_manual_clock(fpas_compiler::compile(&program).expect("compile"))
            .expect("session");
    assert!(matches!(
        session.continue_execution().expect("run"),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn selection_rejects_a_case_moved_to_another_task() {
    let (program, errors) = fpas_parser::parse(
        r#"program WrongOwner;
uses Std.Tasks as Tasks;
function Other(C: Tasks.WaitCase): integer;
begin return Tasks.Select([C]); end function;
begin
  const C: Tasks.WaitCase := Tasks.TimerCase(0, procedure() begin null; end procedure);
  const T: task := go Other(C);
  discard Tasks.Wait(T);
end program;"#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::vm::Vm::new(fpas_compiler::compile(&program).unwrap());
    let error = vm.run().expect_err("wrong owner must fail");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_INVALID_TASK);
    assert!(error.message.contains("task that created it"), "{error:?}");
}
