//! Creating-task terminal publication waits for every remaining group child.

use super::*;

const SOURCE: &str =
    include_str!("../../../../../../../tests/concurrency/task_owner_cleanup_test.fpas");

fn compile(source: &str) -> fpas_bytecode::VerifiedExecutable {
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    fpas_compiler::compile(&program).expect("compile")
}

#[test]
fn parent_cleanup_joins_children_before_result_publication_with_one_worker() {
    let mut vm = crate::vm::Vm::new(compile(SOURCE));
    vm.pool_size = 1;
    vm.run().expect("creating task joined its groups");
}

#[test]
fn debugger_parent_cleanup_has_the_same_publication_boundary() {
    let mut session = DebugSession::with_manual_clock(compile(SOURCE)).expect("session");
    assert!(matches!(
        session.continue_execution().expect("cleanup"),
        DebugRunResult::Terminated(_)
    ));
}

const MAIN: &str = "program MainCleanup; uses Std.Tasks as Tasks, Std.Console as Console;\n    begin const Group := Tasks.CreateTaskGroup();\n    const Gate: channel of boolean := Tasks.CreateChannel(1);\n    discard Tasks.StartTaskInGroup(Group, procedure(Token: Tasks.CancellationToken) begin\n        discard Tasks.ReceiveWithCancellation(Gate, Token);\n        if not Tasks.IsCancellationRequested(Token) then panic('owner did not cancel'); end if;\n        Console.WriteLn('joined');\n    end procedure);\n    end.";

#[test]
fn main_cleanup_executes_children_before_returning() {
    let mut vm = crate::vm::Vm::new(compile(MAIN));
    vm.pool_size = 1;
    vm.run().expect("main cleanup");
    assert_eq!(vm.output().lines, ["joined"]);
}

#[test]
fn debugger_main_cleanup_finishes_children_before_termination() {
    let mut session = DebugSession::with_manual_clock(compile(MAIN)).expect("session");
    assert!(matches!(
        session.continue_execution().expect("cleanup"),
        DebugRunResult::Terminated(_)
    ));
    let events = session.take_task_events();
    assert_eq!(
        events
            .iter()
            .filter(|event| event.kind == DebugTaskEventKind::Exited)
            .count(),
        1
    );
}

#[test]
fn dynamically_received_callable_cannot_authorize_foreign_global_access() {
    let source = "program DynamicWorker; uses Std.Tasks as Tasks;\n        var Counter: integer := 1;\n        function Read(): integer; begin return Counter; end function;\n        begin const Queue: channel of function(): integer := Tasks.CreateChannel(1);\n        discard Tasks.Send(Queue, Read);\n        const Work := Tasks.Receive(Queue).Unwrap();\n        scope discard go Work(); end scope; end.";
    let mut vm = crate::vm::Vm::new(compile(source));
    vm.pool_size = 1;
    let error = vm
        .run()
        .expect_err("unknown target retains mandatory VM checks");
    assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_INVALID_TASK);
    assert!(error.message.contains("mutable program or unit global"));
    let mut session = DebugSession::with_manual_clock(compile(source)).unwrap();
    let stop = stopped(session.continue_execution().unwrap());
    assert_eq!(stop.diagnostic.as_ref().unwrap().code, error.code);
    assert_eq!(stop.diagnostic.as_ref().unwrap().span, error.span);
}

#[test]
fn forced_main_entry_keeps_cleanup_pending_without_dispatching_children() {
    let source = MAIN
        .replace(
            "Std.Console as Console;",
            "Std.Console as Console, Std.Time as Time;",
        )
        .replace(
            "begin const Group",
            "procedure Finish(); begin Time.Sleep(0); end procedure; begin const Group",
        )
        .replace("    end.", "    Finish(); end.");
    let mut session = DebugSession::with_manual_clock(compile(&source)).unwrap();
    session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "Finish".into(),
        }])
        .unwrap();
    let stop = stopped(session.continue_execution().unwrap());
    assert_eq!(stop.task_id, 0);
    let stack = session.stack(0, 8).unwrap();
    let entry = stack.items.last().unwrap().id;
    let before = session.test_instruction_count();
    let result = session.force_return(entry, None).unwrap();
    assert!(!result.terminated, "group children are still pending");
    assert_eq!(
        session.test_instruction_count(),
        before,
        "forced completion never executes child code inline"
    );
    assert_eq!(
        session
            .tasks(0, 8)
            .unwrap()
            .items
            .into_iter()
            .find(|task| task.id == 0)
            .unwrap()
            .state,
        DebugTaskState::Waiting
    );
    session.replace_function_breakpoints(Vec::new()).unwrap();
    assert!(matches!(
        session.continue_execution().unwrap(),
        DebugRunResult::Terminated(_)
    ));
    assert_eq!(session.output().lines, ["joined"]);
}

#[test]
fn cancellation_cannot_hide_an_unexpected_child_failure_during_group_cleanup() {
    let source = "program CancelOwner; uses Std.Tasks as Tasks, Std.Time as Time;\n        procedure Finish(); begin Time.Sleep(0); end procedure;\n        function Parent(): integer;\n        begin const Group := Tasks.CreateTaskGroup();\n        const Gate: channel of boolean := Tasks.CreateChannel(1);\n        discard Tasks.StartTaskInGroup(Group, procedure(Token: Tasks.CancellationToken) begin\n            discard Tasks.ReceiveWithCancellation(Gate, Token); panic('cleanup panic');\n        end procedure);\n        Finish(); return 42; end function;\n        begin scope discard go Parent(); end scope; end.";
    let mut session = DebugSession::with_manual_clock(compile(source)).unwrap();
    session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "Finish".into(),
        }])
        .unwrap();
    let stop = stopped(session.continue_execution().unwrap());
    assert_eq!(stop.task_id, 1);
    session.cancel_task(1).unwrap();
    session.replace_function_breakpoints(Vec::new()).unwrap();
    let stop = stopped(session.continue_execution().unwrap());
    let diagnostic = stop.diagnostic.unwrap();
    assert_eq!(
        diagnostic.code,
        fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC
    );
    assert!(diagnostic.message.contains("cleanup panic"));
}

#[test]
fn cancellation_preserves_an_already_terminal_group_child_failure() {
    let source = "program CancelCompletedChild; uses Std.Tasks as Tasks, Std.Time as Time;\n        procedure Finish(); begin Time.Sleep(0); end procedure;\n        function Parent(): integer;\n        begin const Group := Tasks.CreateTaskGroup();\n        discard Tasks.StartTaskInGroup(Group, procedure(Token: Tasks.CancellationToken) begin\n            panic('already failed child');\n        end procedure);\n        discard Tasks.Select([Tasks.TimerCase(5, procedure() begin end procedure)]);\n        Finish(); return 42; end function;\n        begin scope discard go Parent(); end scope; end.";
    let mut session = DebugSession::with_manual_clock(compile(source)).unwrap();
    session
        .replace_function_breakpoints(vec![FunctionBreakpoint {
            name: "Finish".into(),
        }])
        .unwrap();
    let stop = stopped(session.continue_execution().unwrap());
    assert_eq!(stop.task_id, 1);
    assert_eq!(
        session.tasks(0, 8).unwrap().items[2].state,
        DebugTaskState::Failed
    );
    let before = session.test_instruction_count();
    session.cancel_task(1).unwrap();
    assert_eq!(session.test_instruction_count(), before);
    assert_eq!(
        session.tasks(0, 8).unwrap().items[1].state,
        DebugTaskState::Failed,
        "immediate cleanup reports the child failure instead of cancellation"
    );
    session.replace_function_breakpoints(Vec::new()).unwrap();
    let diagnostic = stopped(session.continue_execution().unwrap())
        .diagnostic
        .unwrap();
    assert_eq!(
        diagnostic.code,
        fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC
    );
    assert!(diagnostic.message.contains("already failed child"));
}

#[test]
fn identical_group_child_diagnostics_remain_independent_failures() {
    let source = "program DuplicateCleanup; uses Std.Tasks as Tasks, Std.Time as Time;\n        procedure Fail(Token: Tasks.CancellationToken); begin panic('same child failure'); end procedure;\n        begin const Group := Tasks.CreateTaskGroup();\n        const A := Tasks.StartTaskInGroup(Group, Fail);\n        discard Tasks.StartTaskInGroup(Group, Fail);\n        discard Tasks.Select([Tasks.TimerCase(5, procedure() begin end procedure)]);\n        Tasks.Wait(A); end.";
    let check = |error: crate::vm::VmError| {
        assert_eq!(error.code, fpas_diagnostics::codes::RUNTIME_PROGRAM_PANIC);
        let help = error.help.unwrap();
        assert_eq!(
            help.matches("Further failure during task group cleanup:")
                .count(),
            1
        );
        assert!(help.contains("Task 2"), "{help}");
        assert!(
            !help.contains("Task 1"),
            "observed child is not reported twice: {help}"
        );
    };
    let mut vm = crate::vm::Vm::new(compile(source));
    vm.pool_size = 1;
    check(vm.run().expect_err("both children failed"));
    let mut session = DebugSession::with_manual_clock(compile(source)).unwrap();
    check(
        stopped(session.continue_execution().unwrap())
            .diagnostic
            .unwrap(),
    );
}
