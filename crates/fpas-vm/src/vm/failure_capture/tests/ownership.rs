//! Original task causes through waits, scope exit, and group cleanup.

use super::*;

#[test]
fn scheduled_callback_result_failure_uses_the_restored_hosted_callsite() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/stdlib/str/map_multiple_result_in_task_runtime_error.fpas"
    ));
    let expected = vm(source).run().unwrap_err();
    let mut vm = vm(source);
    let output = observe(&mut vm);
    let error = vm.run().unwrap_err();
    assert_eq!(
        error, expected,
        "capture does not alter ordinary diagnostics"
    );
    let output = output.lock().unwrap();
    let frame = &output[0].cause.frames[0];
    assert_eq!(frame.function.to_ascii_lowercase(), "work");
    assert_eq!(frame.location.unwrap().line, 12);
    assert_eq!(frame.reason, None);
}

#[test]
fn worker_causes_survive_wait_scope_exit_and_owner_group_cleanup() {
    let cases = [
        "scope const Child := go Work(Ready); discard Tasks.Wait(Child); end scope;",
        "scope discard go Work(Ready); discard Tasks.Receive(Ready); end scope;",
        "const Group := Tasks.CreateTaskGroup(); const Child := Tasks.StartTaskInGroup(Group, function(Token: Tasks.CancellationToken): integer begin return Work(Ready); end function); discard Tasks.WaitAny([Child]);",
    ];
    for body in cases {
        let source = format!(
            "program WorkerCause; uses Std.Tasks as Tasks;\nfunction Work(Ready: channel of integer): integer; begin const Before := 37; discard Tasks.Send(Ready, 1); panic('worker cause'); end function;\nbegin const Ready: channel of integer := Tasks.CreateChannel(1); {body} end."
        );
        let mut vm = vm(&source);
        vm.pool_size = 1;
        let output = observe(&mut vm);
        let error = vm.run().expect_err("worker failure escapes");
        let output = output.lock().unwrap();
        assert_eq!(output.len(), 1, "{body}");
        let data = &output[0];
        assert_eq!(data.error.diagnostic, error);
        assert_ne!(data.cause.task_id, Some(0));
        assert_eq!(data.cause.frames[0].function.to_ascii_lowercase(), "work");
        assert_eq!(
            scalar(&data.cause.frames[0], "Before"),
            &Some(FailureScalar::Integer("37".into()))
        );
        assert!(
            data.related_errors.is_empty(),
            "wait must not duplicate the child cause"
        );
    }
}

#[test]
fn body_failure_keeps_group_cleanup_failure_as_a_structured_secondary() {
    let mut vm = vm(
        "program Secondary; uses Std.Tasks as Tasks; begin const Ready: channel of integer := Tasks.CreateChannel(1); const Group := Tasks.CreateTaskGroup(); discard Tasks.StartTaskInGroup(Group, procedure(Token: Tasks.CancellationToken) begin const ChildLocal := 37; discard Tasks.Send(Ready, 1); panic('child'); end procedure); discard Tasks.Receive(Ready); const RootLocal := 19; panic('body'); end.",
    );
    let output = observe(&mut vm);
    let error = vm.run().unwrap_err();
    assert_eq!(error.message, "panic: body");
    assert!(error.help.as_deref().unwrap().contains("panic: child"));
    let output = output.lock().unwrap();
    let data = &output[0];
    assert_eq!(data.cause.task_id, Some(0));
    assert_eq!(
        scalar(&data.cause.frames[0], "RootLocal"),
        &Some(FailureScalar::Integer("19".into()))
    );
    assert_eq!(data.related_errors.len(), 1);
    let child = &data.related_errors[0];
    assert_ne!(child.cause.task_id, Some(0));
    assert_eq!(child.error.message, "panic: child");
    assert_eq!(
        scalar(&child.cause.frames[0], "ChildLocal"),
        &Some(FailureScalar::Integer("37".into()))
    );
}
