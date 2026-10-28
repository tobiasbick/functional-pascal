//! Send payloads cross domains, but selection callbacks remain on their owner.

use super::*;
use crate::FunctionId;
use crate::execution::vm::channels::{ReceiveState, SendState};
use crate::execution::vm::tasks::TaskScheduler;
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;
use std::sync::Arc;

fn worker() -> (Worker, crate::FunctionTarget) {
    let (program, errors) = fpas_parser::parse(
        "program SelectionOwnership;
         function Make(): procedure(Outcome: result of (boolean, string));
         begin
           var Count := 0;
           return procedure(Outcome: result of (boolean, string))
             begin Count := Count + 1; end procedure;
         end function;
         begin end.",
    );
    assert!(errors.is_empty(), "{errors:?}");
    let image = fpas_compiler::compile(&program).expect("image");
    let callback = image
        .executable()
        .functions
        .iter()
        .position(|info| info.arity == 1 && info.capture_count == 1)
        .expect("capturing callback");
    let target = image
        .function_target(FunctionId::new(callback as u16))
        .expect("target");
    let worker = Worker::new(Arc::new(image))
        .expect("worker")
        .with_scheduler(Some(Arc::new(TaskScheduler::new())));
    (worker, target)
}

#[test]
fn send_case_rejects_local_payload_but_accepts_and_executes_local_callback() {
    let (mut worker, target) = worker();
    let queue = worker.hosted.channels.create(2).expect("channel");
    let count = worker
        .domain
        .new_cell(Value::Integer(0))
        .expect("callback cell");
    let callback = Value::task_owned_function(
        target,
        vec![Value::Cell(count.clone())],
        worker.domain.id().clone(),
    );
    let _observation = worker
        .domain
        .root_values(crate::RootKind::Host, vec![Value::Cell(count.clone())]);
    for payload in [
        Value::Cell(count.clone()),
        Value::option_some(Value::Cell(count.clone())),
    ] {
        let error = worker
            .selection_intrinsic(
                Intrinsic::Task(TaskIntrinsic::SendCase),
                &[Value::OpaqueHandle(queue), payload, callback.clone()],
                None,
            )
            .expect_err("local send payload");
        assert_eq!(error.code, RUNTIME_INVALID_TASK);
        assert!(matches!(
            worker.hosted.channels.receive(queue, false, None),
            Ok(ReceiveState::Pending)
        ));
        assert_eq!(worker.hosted.cases.close_owner(worker.task_id), 0);
    }
    let arguments = [Value::OpaqueHandle(queue), Value::Integer(42), callback];
    let Some(Some(Value::OpaqueHandle(closed))) = worker
        .selection_intrinsic(Intrinsic::Task(TaskIntrinsic::SendCase), &arguments, None)
        .expect("local callback retained")
    else {
        panic!("WaitCase handle expected");
    };
    assert_eq!(worker.hosted.cases.close_owner(worker.task_id), 1);
    assert_eq!(
        count.read(&worker.domain).expect("counter"),
        Value::Integer(0),
        "cleanup must not execute callback"
    );
    assert!(
        !worker
            .hosted
            .cases
            .close(worker.task_id, closed)
            .expect("closed handle")
    );
    assert!(
        matches!(
            worker
                .hosted
                .channels
                .send(queue, Value::Integer(7), false, None),
            Ok(SendState::Sent)
        ),
        "case cleanup must not close the channel"
    );
    let Some(Some(Value::OpaqueHandle(selected))) = worker
        .selection_intrinsic(Intrinsic::Task(TaskIntrinsic::SendCase), &arguments, None)
        .expect("retry callback retained")
    else {
        panic!("WaitCase handle expected");
    };
    assert_ne!(closed, selected);
    worker
        .selection_intrinsic(
            Intrinsic::Task(TaskIntrinsic::Select),
            &[Value::Array(vec![Value::OpaqueHandle(selected)].into())],
            None,
        )
        .expect("select");
    worker.run_in_place().expect("local callback invocation");
    assert_eq!(
        count.read(&worker.domain).expect("counter"),
        Value::Integer(1)
    );
    assert!(matches!(
        worker.hosted.channels.receive(queue, false, None),
        Ok(ReceiveState::Received(Value::Integer(7)))
    ));
    assert!(matches!(
        worker.hosted.channels.receive(queue, false, None),
        Ok(ReceiveState::Received(Value::Integer(42)))
    ));
}
