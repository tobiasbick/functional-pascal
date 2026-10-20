//! Controlled completion barriers in the normal VM and deterministic debugger.

use super::*;

const SOURCE: &str = r#"program ControlledWaitAny;
uses Std.Tasks as Tasks, Std.Time as Time;
function Work(): integer;
begin
  Time.Sleep(30);
  return 7;
end function;
function CancelLater(Source: Tasks.CancellationSource): integer;
begin
  Time.Sleep(1);
  discard Tasks.Cancel(Source);
  return 0;
end function;
begin scope
  const T: task of integer := go Work();
  case Tasks.WaitAnyWithTimeout([T], 0) of
    when Ok(_): panic('pending task ready');
    when Error(const Message): if not (Message is Tasks.TaskError.Timeout('Std.Tasks.WaitAnyWithTimeout')) then panic(Message.Message()); end if;
  end case;
  case Tasks.WaitAnyWithTimeout([T], 1) of
    when Ok(_): panic('deadline extended');
    when Error(const Message): if not (Message is Tasks.TaskError.Timeout('Std.Tasks.WaitAnyWithTimeout')) then panic(Message.Message()); end if;
  end case;
  const Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();
  const Canceller: task of integer := go CancelLater(Source);
  case Tasks.WaitAnyWithCancellation([T], Tasks.GetCancellationToken(Source)) of
    when Ok(_): panic('not cancelled');
    when Error(const Message): if not (Message is Tasks.TaskError.Cancelled('Std.Tasks.WaitAnyWithCancellation')) then panic(Message.Message()); end if;
  end case;
  if Tasks.Wait(Canceller) <> 0 then panic('canceller result'); end if;
  if Tasks.Wait(T) <> 7 then panic('result lost'); end if;
  case Tasks.WaitAnyWithTimeout([T], 0) of
    when Ok(const Index): if Index <> 0 then panic('index'); end if;
    when Error(const Message): panic(Message.Message());
  end case;
  case Tasks.WaitAnyWithCancellation([T], Tasks.GetCancellationToken(Source)) of
    when Ok(_): panic('pre-cancellation lost');
    when Error(const Message): if not (Message is Tasks.TaskError.Cancelled('Std.Tasks.WaitAnyWithCancellation')) then panic(Message.Message()); end if;
  end case;
  const Active: Tasks.CancellationSource := Tasks.CreateCancellationSource();
  case Tasks.WaitAnyWithCancellation([T], Tasks.GetCancellationToken(Active)) of
    when Ok(const Index): if Index <> 0 then panic('index'); end if;
    when Error(const Message): panic(Message.Message());
  end case;
 end scope; end."#;

#[test]
fn controlled_wait_any_uses_debugger_deadlines_and_cancellation() {
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
fn controlled_wait_any_preserves_results_with_one_worker() {
    let (program, errors) = fpas_parser::parse(
        r#"program ReadyControlledWait;
uses Std.Tasks as Tasks;
function Work(): integer;
begin
  return 7;
end function;
begin scope
  const T: task of integer := go Work();
  Tasks.WaitAll([T]);
  case Tasks.WaitAnyWithTimeout([T], 0) of
    when Ok(const Index): if Index <> 0 then panic('index'); end if;
    when Error(const Message): panic(Message.Message());
  end case;
  const Source: Tasks.CancellationSource := Tasks.CreateCancellationSource();
  discard Tasks.Cancel(Source);
  case Tasks.WaitAnyWithCancellation([T], Tasks.GetCancellationToken(Source)) of
    when Ok(_): panic('pre-cancellation lost');
    when Error(const Message): if not (Message is Tasks.TaskError.Cancelled('Std.Tasks.WaitAnyWithCancellation')) then panic(Message.Message()); end if;
  end case;
  if Tasks.Wait(T) <> 7 then panic('result consumed'); end if;
 end scope; end."#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::vm::Vm::new(fpas_compiler::compile(&program).expect("compile"));
    vm.pool_size = 1;
    vm.run().expect("controlled waits");
}
