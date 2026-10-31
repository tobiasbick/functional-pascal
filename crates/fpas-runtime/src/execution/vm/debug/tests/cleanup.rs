//! Debug cancellation drives resource release rather than discarding frames.

use super::*;

#[test]
fn disconnect_runs_a_suspending_release() {
    let source = r#"program T;
uses Std.Console as Console, Std.Time as Time;
type Resource = record
  release procedure Close(Self: Resource);
  begin Time.Sleep(5); Console.WriteLn('released'); end procedure;
end record;
begin
  using const Item := Resource();
  Console.WriteLn('body');
end."#;
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let mut session =
        DebugSession::new(fpas_compiler::compile(&program).expect("compile")).expect("session");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".into(),
            line: 9,
            column: None,
        })
        .unwrap();
    stopped(session.continue_execution().expect("breakpoint"));
    let frame = session.stack(0, 1).unwrap().items[0].id;
    let error = session
        .force_return(frame, None)
        .expect_err("entry resource blocks forced return");
    assert!(error.message.contains("resource"), "{error:?}");
    session.disconnect();
    assert_eq!(session.output().lines, ["released"]);
}

#[test]
fn cancel_task_defers_release_until_resume_and_can_suspend() {
    let source = r#"program T;
uses Std.Console as Console, Std.Time as Time;
type Resource = record
  release procedure Close(Self: Resource);
  begin Time.Sleep(5); Console.WriteLn('released'); end procedure;
end record;
procedure Child();
begin
  using const Item := Resource();
  Console.WriteLn('body');
  Time.Sleep(100000);
end procedure;
begin scope
  discard go Child();
  Time.Sleep(10);
end scope; end."#;
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let mut session =
        DebugSession::new(fpas_compiler::compile(&program).expect("compile")).expect("session");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".into(),
            line: 10,
            column: None,
        })
        .unwrap();
    let stop = stopped(session.continue_execution().expect("breakpoint"));
    assert_eq!(stop.task_id, 1);
    session.cancel_task(1).expect("cancel child");
    assert!(session.output().lines.is_empty());
    let result = session.continue_execution();
    assert_eq!(
        session.output().lines,
        ["released"],
        "resume result: {result:?}"
    );
}

#[test]
fn frame_replacement_cannot_discard_a_live_resource() {
    let source = r#"program T;
uses Std.Console as Console;
type Resource = record
  release procedure Close(Self: Resource);
  begin Console.WriteLn('released'); end procedure;
end record;
procedure Work();
begin
  using const Item := Resource();
  Console.WriteLn('body');
end procedure;
begin Work(); end."#;
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let mut session =
        DebugSession::new(fpas_compiler::compile(&program).expect("compile")).expect("session");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".into(),
            line: 10,
            column: None,
        })
        .unwrap();
    stopped(session.continue_execution().expect("breakpoint"));
    let frame = session.stack(0, 1).unwrap().items[0].id;
    let restart = session
        .restart_frame(frame)
        .expect_err("live resource blocks restart");
    assert!(restart.message.contains("resource"), "{restart:?}");
    let forced = session
        .force_return(frame, None)
        .expect_err("live resource blocks forced return");
    assert!(forced.message.contains("resource"), "{forced:?}");
    assert!(matches!(
        session.continue_execution().unwrap(),
        DebugRunResult::Terminated(_)
    ));
    assert_eq!(session.output().lines, ["body", "released"]);
}

#[test]
fn frame_replacement_cannot_discard_an_open_scope_before_acquisition() {
    let source = r#"program T;
uses Std.Console as Console;
type Resource = record
  release procedure Close(Self: Resource);
  begin Console.WriteLn('released'); end procedure;
end record;
procedure Work();
begin
  Console.WriteLn('before');
  using const Item := Resource();
end procedure;
begin Work(); end."#;
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    let mut session =
        DebugSession::new(fpas_compiler::compile(&program).expect("compile")).expect("session");
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".into(),
            line: 9,
            column: None,
        })
        .unwrap();
    stopped(session.continue_execution().expect("breakpoint"));
    let frame = session.stack(0, 1).unwrap().items[0].id;
    assert!(session.restart_frame(frame).is_err());
    assert!(session.force_return(frame, None).is_err());
    assert!(matches!(
        session.continue_execution().unwrap(),
        DebugRunResult::Terminated(_)
    ));
    assert_eq!(session.output().lines, ["before", "released"]);
}
