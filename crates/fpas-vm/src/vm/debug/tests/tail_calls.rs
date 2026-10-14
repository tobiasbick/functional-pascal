//! Debugger-owned execution keeps every frame of direct tail calls.

use super::*;

const SOURCE: &str = "program TailDebug;\n\nuses Std.Console as Console;\n\nfunction Count(N: integer): integer;\nbegin\n  if N = 0 then\n  begin\n    return 0;\n  end; end if;\n\n  return Count(N - 1);\nend function;\n\nbegin\n  Console.WriteLn(Count(3));\nend.\n";

#[test]
fn debug_stacks_show_each_tail_call_frame() {
    let (program, diagnostics) = fpas_parser::parse(SOURCE);
    assert!(diagnostics.is_empty(), "parse diagnostics: {diagnostics:?}");
    let executable = fpas_compiler::compile(&program).expect("compile tail-call fixture");
    assert!(
        executable
            .executable()
            .code
            .iter()
            .any(|word| word.opcode() == Ok(Opcode::TailCall)),
        "the recursive return must compile to a tail call"
    );
    let mut session = DebugSession::new(executable).expect("debug session");
    let breakpoint = session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".to_string(),
            line: 9,
            column: None,
        })
        .expect("breakpoint");
    assert!(breakpoint.is_verified());
    let stop = stopped(session.continue_execution().expect("continue to base case"));
    assert_eq!(stop.reason, DebugStopReason::Breakpoint);
    let frames = session.stack(0, 16).expect("stack").items;
    assert_eq!(
        frames.iter().filter(|frame| frame.name == "count").count(),
        4,
        "Count(3) .. Count(0) stay visible while debugging"
    );
    assert!(matches!(
        session.continue_execution().expect("finish"),
        DebugRunResult::Terminated(_)
    ));
    assert_eq!(session.output().lines, ["0"]);
}
