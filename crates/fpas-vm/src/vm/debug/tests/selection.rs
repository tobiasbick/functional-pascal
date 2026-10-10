//! Mixed-source selection uses the same resumable callbacks in the VM and debugger.

use super::*;

const SOURCE: &str = r#"program SelectionContinuation;
uses Std.Tasks, Std.Time;
function Produce(Q: channel of integer): integer;
begin
  Sleep(2);
  discard Send(Q, 123);
  return 1;
end function;
function Child(): integer;
begin
  Sleep(2);
  return 7;
end function;
function Parent(): integer;
begin
  const T: task := go Child();
  var Seen: integer := 0;
  const C: WaitCase := TaskCase(T, procedure()
  begin
    Sleep(0);
    Sleep(1);
    Seen := Wait(T);
  end procedure);
  if Select([C]) <> 0 then panic('task index'); end if;
  return Seen;
end function;
begin
  const T: task := go Parent();
  const Q: channel of integer := CreateChannel(1);
  var Seen: integer := 0;
  const S: WaitCase := SendCase(Q, 42, procedure(R: result of (boolean, string))
  begin
    if not R.Unwrap() then panic('send result'); end if;
    Sleep(1);
    const Receive: WaitCase := ReceiveCase(Q, procedure(V: result of (integer, string))
    begin Seen := V.Unwrap(); end procedure);
    if Select([Receive]) <> 0 then panic('nested selection'); end if;
  end procedure);
  if Select([S]) <> 0 then panic('send index'); end if;
  if Seen <> 42 then panic('callback did not complete'); end if;
  if Wait(T) <> 7 then panic('task value lost'); end if;
  const Producer: task := go Produce(Q);
  const Pending: WaitCase := ReceiveCase(Q, procedure(R: result of (integer, string))
  begin Seen := R.Unwrap(); end procedure);
  const Fallback: WaitCase := TimerCase(1000, procedure() begin panic('pending receive timed out'); end procedure);
  if Select([Pending, Fallback]) <> 0 then panic('pending receive index'); end if;
  if Seen <> 123 then panic('pending receive value'); end if;
  if Wait(Producer) <> 1 then panic('producer result'); end if;
  const Closed: WaitCase := ReceiveCase(Q, procedure(R: result of (integer, string))
  begin
    case R of
      when Ok(_): panic('closed channel delivered');
      when Error(const Message): if Message <> 'Channel is closed' then panic(Message); end if;
    end case;
  end procedure);
  discard CloseChannel(Q);
  if Select([Closed]) <> 0 then panic('closed index'); end if;
  const Timer: WaitCase := TimerCase(2, procedure() begin Seen := 99; end procedure);
  if Select([Timer]) <> 0 then panic('timer index'); end if;
  if Seen <> 99 then panic('timer callback'); end if;
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
uses Std.Tasks;
function Other(C: WaitCase): integer;
begin return Select([C]); end function;
begin
  const C: WaitCase := TimerCase(0, procedure() begin end procedure);
  const T: task := go Other(C);
  discard Wait(T);
end."#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::vm::Vm::new(fpas_compiler::compile(&program).unwrap());
    let error = vm.run().expect_err("wrong owner must fail");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_INVALID_TASK);
    assert!(error.message.contains("task that created it"), "{error:?}");
}
