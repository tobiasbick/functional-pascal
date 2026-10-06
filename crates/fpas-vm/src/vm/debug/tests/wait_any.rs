//! Deterministic task-completion barriers preserve typed results.

use super::*;

#[test]
fn wait_any_resumes_after_sleep_and_preserves_results() {
    let source = r#"program DebugWaitAny;
uses Std.Tasks, Std.Time;
function Work(Value: integer): integer;
begin
  Sleep(10);
  return Value;
end function;
begin
  var A: task := go Work(11);
  var B: task := go Work(22);
  var Winner: integer := WaitAny([A, B]);
  if (Winner < 0) or (Winner > 1) then panic('index'); end if;
  WaitAll([A, B]);
  if WaitAny([B, A]) <> 0 then panic('order'); end if;
  if Wait(B) <> 22 then panic('consumed B'); end if;
  if WaitAny([B, A]) <> 0 then panic('consumed identity'); end if;
  if Wait(A) <> 11 then panic('consumed A'); end if;
end."#;
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let mut session =
        DebugSession::with_manual_clock(fpas_compiler::compile(&program).expect("compile"))
            .expect("session");
    assert!(matches!(
        session.continue_execution().expect("run"),
        DebugRunResult::Terminated(_)
    ));
}
