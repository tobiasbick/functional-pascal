//! Explicit debugger entry completion is terminal even for retryable result values.

use super::*;

#[test]
fn forcing_a_supervised_error_result_finishes_the_task_without_retrying() {
    let (program, errors) = fpas_parser::parse(
        r#"program ForcedSupervisor;
uses Std.Task, Std.Arrays;
function Work(Token: CancellationToken): result of integer, string;
begin panic('worker body must not execute') end;
begin
  var Group: TaskGroup := CreateTaskGroup();
  var Child: task := StartSupervisedTask(Group, Work, 1023, 0);
  case Wait(Child) of
    Ok(_): panic('forced error lost');
    Error(Message): if Message <> 'forced' then panic('wrong forced error')
  end;
  var Failures: array of TaskFailure := CloseTaskGroup(Group);
  if Length(Failures) <> 1 then panic('wrong report count');
  if Failures[0].Kind <> TaskFailureKind.ReturnedError then panic('wrong report kind');
  if Failures[0].Message <> 'forced' then panic('wrong report message')
end."#,
    );
    assert!(errors.is_empty(), "{errors:?}");
    let mut session =
        DebugSession::with_manual_clock(fpas_compiler::compile(&program).expect("compile"))
            .expect("session");
    let breakpoints = session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "Work".into(),
        }])
        .expect("breakpoint");
    assert!(breakpoints[0].is_verified());
    let stop = stopped(session.continue_execution().expect("stop before worker"));
    assert_eq!(stop.task_id, 1);
    let frame = session.stack(0, 1).expect("worker entry").items[0].id;
    let before = session.test_instruction_count();
    session
        .force_return(
            frame,
            Some(&DebugExpression::ResultError(Box::new(
                DebugExpression::String("forced".into()),
            ))),
        )
        .expect("force terminal result");
    assert_eq!(session.test_instruction_count(), before);
    assert!(matches!(
        session.continue_execution().expect("collect forced report"),
        DebugRunResult::Terminated(_)
    ));
    let events = session.take_task_events();
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == crate::DebugTaskEventKind::Started)
            .count(),
        1
    );
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == crate::DebugTaskEventKind::Exited)
            .count(),
        1
    );
}
