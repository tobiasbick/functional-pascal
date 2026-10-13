//! Scope completion and nested cancellation in the deterministic task driver.

use super::*;
use fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC;

fn session(source: &str) -> DebugSession {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    DebugSession::with_manual_clock(fpas_compiler::compile(&program).expect("compile"))
        .expect("session")
}

#[test]
fn normal_scope_exit_awaits_both_go_forms_in_debugger() {
    let mut session = session(
        "program Complete;
uses Std.Tasks, Std.Time;
procedure SendLater(Q: channel of integer; N: integer);
begin Sleep(2); discard Send(Q, N); end procedure;
begin
 const Q: channel of integer := CreateChannel(2);
 scope discard go SendLater(Q, 1); const Child: task := go SendLater(Q, 2); end scope;
 for Index: integer := 1 to 2 do
   case TryReceive(Q) of
     when Ok(Some(_)): null;
     when Ok(None): panic('scope ended before its children');
     when Error(_): panic('closed');
   end case;
 end for;
end.",
    );
    assert!(matches!(
        session.continue_execution().expect("run"),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn early_return_cancels_nested_scope_children_before_debugger_completion() {
    let mut session = session(
        "program CancelNested;
uses Std.Tasks, Std.Time;
procedure Child(Q: channel of integer; Started: channel of integer);
begin discard Send(Started, 1); discard Receive(Q); end procedure;
procedure Parent(Q: channel of integer; Started: channel of integer);
begin scope discard go Child(Q, Started); discard Receive(Q); end scope; end procedure;
function Run(Q: channel of integer; Started: channel of integer): integer;
begin scope discard go Parent(Q, Started); discard Receive(Started); return 7; end scope; end function;
procedure Done(); begin Sleep(0); end procedure;
begin const Q: channel of integer := CreateChannel(1);
 const Started: channel of integer := CreateChannel(1);
 if Run(Q, Started) <> 7 then panic('return'); end if;
 Done();
end.",
    );
    let bound = session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "Done".into(),
        }])
        .expect("breakpoint");
    assert!(bound[0].is_verified());
    let stop = stopped(session.continue_execution().expect("run"));
    assert_eq!(stop.reason, DebugStopReason::Breakpoint);
    let tasks = session.tasks(0, 8).expect("tasks");
    assert!(
        tasks
            .items
            .iter()
            .filter(|task| task.id != 0)
            .all(|task| task.state == DebugTaskState::Cancelled)
    );
    assert_eq!(
        tasks.items.len(),
        3,
        "parent and grandchild must both have started"
    );
    assert!(matches!(
        session.continue_execution().expect("finish"),
        DebugRunResult::Terminated(_)
    ));
}

#[test]
fn debugger_task_cancel_joins_open_scope_children_before_publishing_completion() {
    let template = "program CancelParent;
uses Std.Tasks, Std.Time;
procedure BeforeWait(); begin Sleep(0); end procedure;
procedure Child(Q: channel of integer); begin discard Receive(Q); end procedure;
procedure Parent(Q: channel of integer);
begin scope discard go Child(Q); BeforeWait(); discard Receive(Q); end scope; end procedure;
begin const Q: channel of integer := CreateChannel(1); scope discard go Parent(Q); end scope; end.";
    for source in [
        template.to_string(),
        template.replace(
            "scope discard go Parent(Q); end scope;",
            "const Group := CreateTaskGroup(); discard StartTaskInGroup(Group, procedure(Token: CancellationToken) begin Parent(Q); end procedure); scope Sleep(50); end scope; discard CloseTaskGroup(Group);",
        ),
    ] {
        let (program, errors) = fpas_parser::parse(&source);
        assert!(errors.is_empty(), "{errors:?}");
        let mut session = DebugSession::with_limits(
            fpas_compiler::compile(&program).expect("compile"),
            Vec::new(),
            DebugInspectionLimits::default(),
            DebugExecutionLimits {
                timeout: std::time::Duration::from_secs(1),
                ..DebugExecutionLimits::default()
            },
        )
        .expect("session");
        let bound = session
            .replace_function_breakpoints(vec![FunctionBreakpoint {
                name: "BeforeWait".into(),
            }])
            .expect("breakpoint");
        assert!(bound[0].is_verified());
        let stop = stopped(session.continue_execution().expect("breakpoint"));
        assert_eq!(stop.task_id, 1);
        session.cancel_task(1).expect("cancel parent");
        let tasks = session.tasks(0, 8).expect("tasks");
        assert_eq!(
            tasks
                .items
                .iter()
                .find(|task| task.id == 1)
                .expect("parent")
                .state,
            DebugTaskState::Waiting,
            "a parent cannot complete before its scope children"
        );
        let result = session.continue_execution().expect("join cancelled descendants");
        assert!(matches!(result, DebugRunResult::Terminated(_)), "{result:?}");
    }
}

#[test]
fn debugger_reports_original_child_failure_after_scope_unwind() {
    let mut session = session(
        "program FailScope;
uses Std.Tasks, Std.Time;
procedure Spin(); begin while true do null; end while; end procedure;
procedure Fail(); begin panic('child failed'); end procedure;
begin scope discard go Spin(); discard go Fail(); Sleep(100); end scope; end.",
    );
    let result = session.continue_execution().expect("run");
    let DebugRunResult::Stopped(stop) = result else {
        panic!("failure must stop: {result:?}")
    };
    let error = stop.diagnostic.expect("original diagnostic");
    assert_eq!(error.code, RUNTIME_PROGRAM_PANIC);
    assert!(error.message.contains("child failed"), "{error:?}");
    assert!(!error.help.unwrap_or_default().contains("Further failures"));
    assert!(
        session
            .tasks(0, 8)
            .expect("tasks")
            .items
            .iter()
            .filter(|task| task.id != 0)
            .all(|task| matches!(
                task.state,
                DebugTaskState::Failed | DebugTaskState::Cancelled
            ))
    );
}
