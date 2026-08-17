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

uses Std.Console;

begin
  WriteLn(ReadLn())
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
    assert_eq!(stop.reason, DebugStopReason::RuntimeError);
    let diagnostic = stop.diagnostic.expect("missing-input diagnostic");
    assert!(diagnostic.message.contains("no input available"));
}
