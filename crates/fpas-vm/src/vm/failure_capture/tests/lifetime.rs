//! Handled failures and retries release captured values rather than retaining history.

use super::*;

#[test]
fn handled_group_failures_retries_and_result_data_do_not_notify_or_retain_snapshots() {
    let mut vm = vm(
        "program Handled; uses Std.Tasks as Tasks;\nbegin\n for I: integer := 1 to 40 do\n const Group := Tasks.CreateTaskGroup();\n const Ready: channel of integer := Tasks.CreateChannel(1);\n discard Tasks.StartTaskInGroup(Group, procedure(Token: Tasks.CancellationToken) begin const Before := 'temporary'; discard Tasks.Send(Ready, 1); panic('handled'); end procedure);\n discard Tasks.Receive(Ready);\n discard Tasks.CloseTaskGroup(Group);\n end for;\n const Group := Tasks.CreateTaskGroup();\n const Attempts: channel of integer := Tasks.CreateChannel(3);\n discard Tasks.Send(Attempts, 0); discard Tasks.Send(Attempts, 1); discard Tasks.Send(Attempts, 2);\n const Retried := Tasks.StartSupervisedTask(Group, function(Token: Tasks.CancellationToken): result of (integer, string) begin const Attempt := Tasks.Receive(Attempts).Unwrap(); if Attempt = 0 then return Error('data'); end if; if Attempt = 1 then panic('retry'); end if; return Ok(42); end function, 2, 0);\n discard Tasks.Wait(Retried); discard Tasks.CloseTaskGroup(Group);\nend.",
    );
    let output = observe(&mut vm);
    vm.run().expect("all failures handled");
    assert!(output.lock().unwrap().is_empty());
    let state = vm.scheduler.failure_capture.get().unwrap();
    let usage = state.usage.lock().unwrap();
    assert_eq!(
        (usage.values, usage.bytes),
        (0, 0),
        "handled outcomes retained snapshots"
    );
    assert!(
        state.next_id.load(std::sync::atomic::Ordering::Relaxed) > 40,
        "fixture must actually capture handled failures and the retried panic"
    );
}

#[test]
fn diagnostic_text_is_bounded_without_changing_the_escaping_error() {
    let message = "x".repeat(8192);
    let mut vm = vm(&format!("program Text; begin panic('{message}'); end."));
    let output = observe(&mut vm);
    let error = vm.run().unwrap_err();
    assert_eq!(error.message, format!("panic: {message}"));
    let output = output.lock().unwrap();
    assert_eq!(output[0].error.message.chars().count(), 4096);
    assert_eq!(output[0].error.reason, Some("string_limit"));
}

#[test]
fn diagnostic_clones_own_the_capture_until_the_last_clone_is_released() {
    let mut vm = vm("program Lifetime; begin const Value := 'retained'; panic('failure'); end.");
    assert!(vm.set_failure_observer(|_| {}));
    let error = vm.run().unwrap_err();
    let state = Arc::clone(vm.scheduler.failure_capture.get().unwrap());
    assert!(state.usage.lock().unwrap().values > 0);
    let copy = error.clone();
    drop(error);
    assert!(state.usage.lock().unwrap().values > 0);
    drop(copy);
    assert_eq!(state.usage.lock().unwrap().values, 0);
    assert_eq!(state.usage.lock().unwrap().bytes, 0);
}

#[test]
fn competing_large_causes_share_a_budget_and_keep_identity_when_detail_is_exhausted() {
    use super::super::{MAX_BYTES, MAX_VALUES, identity::Context};
    use crate::vm::worker::Worker;
    use fpas_bytecode::{FunctionId, Value};
    let mut vm = vm(
        "program Bounded; procedure Fail(Items: array of string); begin panic('same failure'); end procedure; begin end.",
    );
    let output = observe(&mut vm);
    let state = Arc::clone(vm.scheduler.failure_capture.get().unwrap());
    let target = vm
        .executable
        .executable()
        .functions
        .iter()
        .position(|info| vm.executable.executable().strings.get(info.name).unwrap() == "fail")
        .unwrap();
    let large = Value::Array(vec![Value::Str("x".repeat(4096).into()); 256].into());
    let mut errors = Vec::new();
    for task in 1..=2000 {
        let mut worker = Worker::for_function_with_state(
            Arc::clone(&vm.executable),
            FunctionId::try_from_index(target).unwrap(),
            vec![large.clone()],
            Arc::clone(&vm.globals),
            vm.layouts.clone().unwrap(),
            Arc::clone(&vm.hosted),
        )
        .unwrap()
        .with_scheduler(Some(Arc::clone(&vm.scheduler)));
        worker.task_id = task;
        errors.push(worker.run_task().unwrap_err());
        let usage = state.usage.lock().unwrap();
        assert!(usage.values <= MAX_VALUES && usage.bytes <= MAX_BYTES);
    }
    let first = errors[0].attachment::<Context>().unwrap().cause.freeze();
    assert!(!first.frames[0].bindings[0].value.children.is_empty());
    let last = errors
        .last()
        .unwrap()
        .attachment::<Context>()
        .unwrap()
        .cause
        .freeze();
    assert_ne!(
        first.id, last.id,
        "equal panic text must not merge independent causes"
    );
    assert_eq!(last.task_id, Some(2000));
    assert_eq!(last.reason, Some("retained_byte_limit"));
    assert_eq!(
        errors[0],
        *errors.last().unwrap(),
        "capture metadata must not change diagnostic equality"
    );
    let mut primary = errors.remove(0);
    super::super::attach_related(&mut primary, &errors);
    vm.notify_failure(&primary);
    {
        let output = output.lock().unwrap();
        assert_eq!(
            output[0].error.message, "panic: same failure",
            "large values must not consume the primary diagnostic allowance"
        );
        assert_eq!(output[0].error.reason, None);
        assert!(output[0].related_errors.len() <= 1024);
        assert!(output[0].related_errors_omitted > 0);
        assert_eq!(
            output[0].related_errors.len() + output[0].related_errors_omitted,
            1999
        );
    }
    {
        let usage = state.usage.lock().unwrap();
        assert!(usage.bytes <= MAX_BYTES && usage.values <= MAX_VALUES);
    }
    drop(primary);
    drop(errors);
    drop(first);
    drop(last);
    output.lock().unwrap().clear();
    drop(output);
    let usage = state.usage.lock().unwrap();
    assert_eq!((usage.values, usage.bytes), (0, 0));
}
