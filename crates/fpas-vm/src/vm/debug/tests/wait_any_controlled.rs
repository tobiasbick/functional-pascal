//! Controlled completion barriers in the normal VM and deterministic debugger.

use super::*;

const SOURCE: &str = r#"program ControlledWaitAny;
uses Std.Tasks, Std.Time;
function Work(): integer;
begin
  Sleep(30);
  return 7;
end function;
function CancelLater(Source: CancellationSource): integer;
begin
  Sleep(1);
  Cancel(Source);
  return 0;
end function;
begin
  var T: task := go Work();
  case WaitAnyWithTimeout([T], 0) of
    Ok(_): panic('pending task ready');
    Error(Message): if Message <> 'Task wait timed out' then panic(Message); end if;
  end;
  case WaitAnyWithTimeout([T], 1) of
    Ok(_): panic('deadline extended');
    Error(Message): if Message <> 'Task wait timed out' then panic(Message); end if;
  end;
  var Source: CancellationSource := CreateCancellationSource();
  var Canceller: task := go CancelLater(Source);
  case WaitAnyWithCancellation([T], GetCancellationToken(Source)) of
    Ok(_): panic('not cancelled');
    Error(Message): if Message <> 'Task wait was cancelled' then panic(Message); end if;
  end;
  if Wait(Canceller) <> 0 then panic('canceller result'); end if;
  if Wait(T) <> 7 then panic('result lost'); end if;
  case WaitAnyWithTimeout([T], 0) of
    Ok(Index): if Index <> 0 then panic('index'); end if;
    Error(Message): panic(Message);
  end;
  case WaitAnyWithCancellation([T], GetCancellationToken(Source)) of
    Ok(_): panic('pre-cancellation lost');
    Error(Message): if Message <> 'Task wait was cancelled' then panic(Message); end if;
  end;
  var Active: CancellationSource := CreateCancellationSource();
  case WaitAnyWithCancellation([T], GetCancellationToken(Active)) of
    Ok(Index): if Index <> 0 then panic('index'); end if;
    Error(Message): panic(Message);
  end;
end."#;

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
uses Std.Tasks;
function Work(): integer;
begin
  return 7;
end function;
begin
  var T: task := go Work();
  WaitAll([T]);
  case WaitAnyWithTimeout([T], 0) of
    Ok(Index): if Index <> 0 then panic('index'); end if;
    Error(Message): panic(Message);
  end;
  var Source: CancellationSource := CreateCancellationSource();
  Cancel(Source);
  case WaitAnyWithCancellation([T], GetCancellationToken(Source)) of
    Ok(_): panic('pre-cancellation lost');
    Error(Message): if Message <> 'Task wait was cancelled' then panic(Message); end if;
  end;
  if Wait(T) <> 7 then panic('result consumed'); end if;
end."#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let mut vm = crate::vm::Vm::new(fpas_compiler::compile(&program).expect("compile"));
    vm.pool_size = 1;
    vm.run().expect("controlled waits");
}
