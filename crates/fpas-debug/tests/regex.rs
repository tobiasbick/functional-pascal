//! Managed patterns retain typed identity in inspection, detached calls and assignment.
#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "debugger fixtures fail locally"
)]
use fpas_runtime::execution::{
    DebugAssignmentTarget, DebugExpression, DebugRunResult, DebugSession, SourceBreakpoint,
};

#[test]
fn compiled_patterns_support_debugger_calls_assignment_and_event_recording() {
    let source = r#"program RegexDebug;
uses Std.Text.Regex as Regex;
function Count(Pattern: Regex.Pattern): integer;
begin
  return Regex.FindAll(Pattern, 'a1 b2', 0).Unwrap().Length();
end function;
begin
  var Pattern := Regex.Compile('[0-9]+', Regex.DefaultOptions()).Unwrap();
  var Copy := Regex.Compile('z', Regex.DefaultOptions()).Unwrap();
  var Before := Count(Pattern);
  Before := Before + 1;
end."#;
    let (program, diagnostics) = fpas_parser::parse(source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let image = fpas_compiler::compile(&program).expect("regex debugger image");
    let mut session = DebugSession::new(image).expect("session");
    session.start_recording();
    session
        .set_breakpoint(SourceBreakpoint {
            source: "<memory>".into(),
            line: 11,
            column: None,
        })
        .expect("breakpoint");
    assert!(matches!(
        session.continue_execution().expect("stop"),
        DebugRunResult::Stopped(_)
    ));
    let frame = session.stack(0, 1).expect("stack").items[0].id;
    let name = |text: &str| DebugExpression::Name(text.into());
    let summary = session
        .evaluate(&name("Pattern"), Some(frame))
        .expect("pattern inspection");
    assert_eq!(summary.type_name, "Std.Text.Regex.Pattern");
    assert!(summary.value.contains("[0-9]+"));
    let count = |argument: &str| DebugExpression::Call {
        callee: Box::new(DebugExpression::Callable("Count".into())),
        arguments: vec![name(argument)],
    };
    assert_eq!(
        session
            .evaluate(&count("Pattern"), Some(frame))
            .expect("detached search")
            .value,
        "2"
    );
    session
        .set_expression(
            &DebugAssignmentTarget {
                root: "Copy".into(),
                selectors: vec![],
            },
            &name("Pattern"),
            Some(frame),
        )
        .expect("share immutable compiled pattern");
    let frame = session.stack(0, 1).expect("stack after edit").items[0].id;
    assert_eq!(
        session
            .evaluate(&count("Copy"), Some(frame))
            .expect("detached copied search")
            .value,
        "2"
    );
    assert!(matches!(
        session.continue_execution().expect("completion"),
        DebugRunResult::Terminated(_)
    ));
    assert!(!session.recording_events().is_empty());
}
