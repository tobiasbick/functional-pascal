//! Pause during hosted work stays cooperative after the intrinsic returns.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use super::*;
use std::time::{Duration, Instant};

fn compile_session(source: &str) -> DebugSession {
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    DebugSession::new(fpas_compiler::compile(&program).expect("compile pause-in-host fixture"))
        .expect("debug session")
}

#[test]
fn pause_does_not_wait_inside_missing_readln() {
    let mut session = compile_session(
        r#"program PauseReadLn;

uses Std.Console as Console;

begin
  Console.WriteLn(Console.ReadLn().Unwrap());
end.
"#,
    );
    session.pause_handle().request_pause();
    let started = Instant::now();
    let stop = stopped(session.continue_execution().expect("missing ReadLn"));
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "ReadLn must not hang waiting for later io.input"
    );
    // A pause may win while the owned input job is still completing. Resuming
    // must report missing queued input rather than waiting for a later io.input.
    let stop = if stop.reason == DebugStopReason::Pause {
        stopped(session.continue_execution().expect("resume missing ReadLn"))
    } else {
        stop
    };
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    let diagnostic = stop.diagnostic.expect("missing-input diagnostic");
    assert!(diagnostic.message.contains("no input available"));
}
