//! Mixed-source selection uses the same resumable callbacks in the VM and debugger.

use super::*;

const SOURCE: &str = r#"program SelectionContinuation;
uses Std.Task, Std.Time, Std.Results;
function Produce(Q: channel of integer): integer;
begin
  Sleep(2);
  Send(Q, 123);
  return 1
end;
function Child(): integer;
begin
  Sleep(2);
  return 7
end;
function Parent(): integer;
begin
  var T: task := go Child();
  mutable var Seen: integer := 0;
  var C: WaitCase := TaskCase(T, procedure()
  begin
    Sleep(0);
    Sleep(1);
    Seen := Wait(T)
  end);
  if Select([C]) <> 0 then panic('task index');
  return Seen
end;
begin
  var T: task := go Parent();
  var Q: channel of integer := CreateChannel(1);
  mutable var Seen: integer := 0;
  var S: WaitCase := SendCase(Q, 42, procedure(R: result of boolean, string)
  begin
    if not Unwrap(R) then panic('send result');
    Sleep(1);
    var Receive: WaitCase := ReceiveCase(Q, procedure(V: result of integer, string)
    begin Seen := Unwrap(V) end);
    if Select([Receive]) <> 0 then panic('nested selection')
  end);
  if Select([S]) <> 0 then panic('send index');
  if Seen <> 42 then panic('callback did not complete');
  if Wait(T) <> 7 then panic('task value lost');
  var Producer: task := go Produce(Q);
  var Pending: WaitCase := ReceiveCase(Q, procedure(R: result of integer, string)
  begin Seen := Unwrap(R) end);
  var Fallback: WaitCase := TimerCase(1000, procedure() begin panic('pending receive timed out') end);
  if Select([Pending, Fallback]) <> 0 then panic('pending receive index');
  if Seen <> 123 then panic('pending receive value');
  if Wait(Producer) <> 1 then panic('producer result');
  var Closed: WaitCase := ReceiveCase(Q, procedure(R: result of integer, string)
  begin
    case R of
      Ok(_): panic('closed channel delivered');
      Error(Message): if Message <> 'Channel is closed' then panic(Message)
    end
  end);
  CloseChannel(Q);
  if Select([Closed]) <> 0 then panic('closed index');
  var Timer: WaitCase := TimerCase(2, procedure() begin Seen := 99 end);
  if Select([Timer]) <> 0 then panic('timer index');
  if Seen <> 99 then panic('timer callback')
end."#;

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
uses Std.Task;
function Other(C: WaitCase): integer;
begin return Select([C]) end;
begin
  var C: WaitCase := TimerCase(0, procedure() begin end);
  var T: task := go Other(C);
  Wait(T)
end."#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::vm::Vm::new(fpas_compiler::compile(&program).unwrap());
    let error = vm.run().expect_err("wrong owner must fail");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_INVALID_TASK);
    assert!(error.message.contains("task that created it"), "{error:?}");
}
