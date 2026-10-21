//! Debugger terminal transitions obey the same publication and case cleanup boundaries.

use std::sync::Arc;

use fpas_bytecode::{FunctionId, Intrinsic, OwnedCell, TaskIntrinsic, Value};

use super::super::TaskSlot;
use super::*;
use crate::vm::TaskResultPoll;
use crate::vm::tasks::{RetainedResultReplacement, TaskClock, TaskScheduler};
use crate::vm::worker::Worker;

fn runtime(retain_child: bool) -> DebugTaskRuntime {
    let (program, errors) = fpas_parser::parse(
        "program Completion; function Make(): procedure(); begin var Count := 0; \
         return procedure() begin Count := Count + 1; end procedure; end function; begin end.",
    );
    assert!(errors.is_empty(), "{errors:?}");
    let image = Arc::new(fpas_compiler::compile(&program).expect("completion fixture"));
    let scheduler = Arc::new(TaskScheduler::new());
    let clock = Arc::new(TaskClock::manual());
    let root = Worker::new(image)
        .expect("root")
        .with_scheduler(Some(Arc::clone(&scheduler)))
        .with_debug_tasks(Arc::clone(&clock));
    let mut child = root.pool_template();
    child.execution = child.domain.execution_roots().ok();
    child.task_id = 1;
    child.retain_result = retain_child;
    if retain_child {
        scheduler.register_result(1);
    }
    let entry_function = child.function;
    let mut runtime = DebugTaskRuntime::new(root, scheduler, clock);
    runtime.tasks.insert(
        1,
        TaskSlot {
            worker: child,
            entry_function,
            name: "child".into(),
            state: DebugTaskState::Runnable,
            failure: None,
            exited: false,
            paused: false,
        },
    );
    runtime
}

fn retain_case(runtime: &mut DebugTaskRuntime, task_id: u64) -> OwnedCell {
    let worker = &mut runtime.tasks.get_mut(&task_id).expect("task").worker;
    let index = worker
        .executable
        .executable()
        .functions
        .iter()
        .position(|info| info.arity == 0 && info.capture_count == 1)
        .expect("mutable callback");
    let target = worker
        .executable
        .function_target(FunctionId::new(index as u16))
        .expect("target");
    let count = worker
        .domain
        .new_cell(Value::Integer(0))
        .expect("callback cell");
    let callback = Value::task_owned_function(
        target,
        vec![Value::Cell(count.clone())],
        worker.domain.id().clone(),
    );
    let handle = worker
        .execute_debug_intrinsic(
            Intrinsic::Task(TaskIntrinsic::TimerCase),
            &[Value::Integer(100), callback],
        )
        .expect("retained local callback");
    assert!(matches!(handle, Value::OpaqueHandle(_)));
    count
}

fn complete(runtime: &mut DebugTaskRuntime, task_id: u64, value: Value) -> Option<bool> {
    let worker = &runtime.tasks.get(&task_id).expect("task").worker;
    runtime.complete_entry(
        task_id,
        worker.function,
        worker.base,
        worker.call_stack.len(),
        value,
    )
}

#[test]
fn forced_child_completion_rejects_local_results_before_any_state_change() {
    for retain_child in [false, true] {
        let mut runtime = runtime(retain_child);
        let count = retain_case(&mut runtime, 1);
        let worker = &runtime.tasks.get(&1).expect("child").worker;
        let instructions = worker.instruction_count;
        let function = worker
            .executable
            .function_target(worker.function)
            .expect("entry target");
        let values = [
            Value::Cell(count.clone()),
            Value::option_some(Value::Cell(count.clone())),
            Value::task_owned_function(function, Vec::new(), worker.domain.id().clone()),
        ];
        for value in values {
            assert_eq!(complete(&mut runtime, 1, value), None);
            let slot = runtime.tasks.get(&1).expect("child remains retained");
            assert_eq!(slot.state, DebugTaskState::Runnable);
            assert!(!slot.exited);
            assert!(slot.failure.is_none());
            assert!(slot.worker.pending_completion.is_none());
            assert_eq!(slot.worker.instruction_count, instructions);
            assert!(runtime.events.is_empty());
        }
        if retain_child {
            assert!(matches!(
                runtime.scheduler.poll_result(1),
                TaskResultPoll::Pending
            ));
        }
        let worker = &runtime.tasks.get(&1).expect("child").worker;
        assert_eq!(
            worker.hosted.cases.close_owner(1),
            1,
            "rejected return retains its active cases"
        );
        assert_eq!(
            count.read(&worker.domain).expect("counter"),
            Value::Integer(0)
        );
    }
}

#[test]
fn forced_completion_releases_cases_for_retained_unretained_and_root_tasks() {
    for (task_id, retain_child) in [(1, true), (1, false), (0, false)] {
        let mut runtime = runtime(retain_child);
        let count = retain_case(&mut runtime, task_id);
        let _observation = runtime
            .tasks
            .get(&task_id)
            .unwrap()
            .worker
            .domain
            .root_values(
                fpas_bytecode::RootKind::Host,
                vec![Value::Cell(count.clone())],
            );
        let value = if task_id == 0 {
            Value::Cell(count.clone())
        } else {
            Value::Integer(42)
        };
        assert_eq!(complete(&mut runtime, task_id, value), Some(task_id == 0));
        let slot = runtime.tasks.get(&task_id).expect("completed task");
        assert_eq!(slot.state, DebugTaskState::Completed);
        assert_eq!(
            slot.worker.hosted.cases.close_owner(task_id),
            0,
            "terminal state retains no unused cases"
        );
        assert_eq!(
            count.read(&slot.worker.domain).expect("counter"),
            Value::Integer(0),
            "cleanup never invokes a callback"
        );
        if task_id == 0 {
            let Some(Value::Cell(returned)) = &runtime.root_result else {
                panic!("root local result remains available");
            };
            assert!(returned.ptr_eq(&count));
        } else if retain_child {
            assert!(matches!(
                runtime.scheduler.poll_result(1),
                TaskResultPoll::Available(Value::Integer(42))
            ));
        }
    }
}

#[test]
fn rejected_debug_result_replacement_preserves_completed_state_and_previous_value() {
    let mut runtime = runtime(true);
    assert_eq!(complete(&mut runtime, 1, Value::Integer(42)), Some(false));
    let worker = &runtime.tasks.get(&1).expect("child").worker;
    let local = Value::option_some(Value::Cell(
        worker.domain.new_cell(Value::Unit).expect("cell"),
    ));
    let events = runtime.events.len();
    assert!(matches!(
        runtime.replace_completed_result(1, local),
        RetainedResultReplacement::Invalid(_)
    ));
    assert_eq!(
        runtime.tasks.get(&1).expect("child").state,
        DebugTaskState::Completed
    );
    assert_eq!(runtime.events.len(), events);
    assert!(matches!(
        runtime.scheduler.poll_result(1),
        TaskResultPoll::Available(Value::Integer(42))
    ));
}

#[test]
fn forced_completion_retains_cases_until_structured_group_cleanup_finishes() {
    let mut runtime = runtime(true);
    let count = retain_case(&mut runtime, 1);
    let weak = count.downgrade();
    drop(count);
    let worker = &mut runtime.tasks.get_mut(&1).expect("child").worker;
    worker.reset_registers(0);
    let group = runtime
        .scheduler
        .groups
        .create(1, || worker.hosted.cancellations.create_owned())
        .expect("owned group");
    runtime
        .scheduler
        .groups
        .enroll(group, 1, 2)
        .expect("grandchild");
    runtime.scheduler.register_result(2);

    assert_eq!(complete(&mut runtime, 1, Value::Integer(42)), Some(false));
    assert_eq!(
        runtime.tasks.get(&1).expect("child").state,
        DebugTaskState::Waiting
    );
    assert!(
        weak.upgrade().is_some(),
        "case stays rooted during pending group cleanup"
    );
    assert!(matches!(
        runtime.scheduler.poll_result(1),
        TaskResultPoll::Pending
    ));

    runtime.scheduler.store_result(2, Value::Unit);
    assert!(runtime.refresh_readiness().expect("group became ready"));
    assert!(matches!(
        runtime.dispatch(1).expect("finish child"),
        super::super::DebugDispatch::Completed {
            task_id: 1,
            main: false
        }
    ));
    assert!(
        weak.upgrade().is_none(),
        "terminal cleanup releases the unused callback"
    );
    assert!(matches!(
        runtime.scheduler.poll_result(1),
        TaskResultPoll::Available(Value::Integer(42))
    ));
}

#[test]
fn disconnect_releases_claimed_selection_callbacks_while_runtime_remains_alive() {
    for task_id in [0, 1] {
        let mut runtime = runtime(true);
        let count = retain_case(&mut runtime, task_id);
        let weak = count.downgrade();
        drop(count);
        let slot = runtime.tasks.get_mut(&task_id).expect("selecting task");
        let handle = slot.worker.registers[0].clone();
        slot.worker
            .execute_debug_intrinsic(
                Intrinsic::Task(TaskIntrinsic::Select),
                &[Value::Array(vec![handle].into())],
            )
            .expect("pending Select");
        assert!(matches!(
            slot.worker.task_suspension,
            Some(crate::vm::tasks::TaskSuspension::Selection(_))
        ));
        slot.state = super::super::state_from_suspension(&slot.worker);
        slot.worker.reset_registers(0);
        assert_eq!(
            slot.worker.hosted.cases.close_owner(task_id),
            0,
            "Select already claimed the case"
        );
        assert!(
            weak.upgrade().is_some(),
            "pending operation retains its callback"
        );

        runtime.cancel();

        assert!(runtime.scheduler.is_shutdown());
        let slot = runtime
            .tasks
            .get(&task_id)
            .expect("disconnected task remains retained");
        assert_eq!(slot.state, DebugTaskState::Cancelled);
        assert!(slot.worker.task_suspension.is_none());
        assert!(
            weak.upgrade().is_none(),
            "disconnect releases claimed callback without waiting for runtime drop"
        );
    }
}

#[test]
fn root_completion_and_shutdown_retire_cancelled_peer_frames_and_cases() {
    for shutdown_sleep in [false, true] {
        let mut runtime = runtime(true);
        let cell = retain_case(&mut runtime, 1);
        let weak = cell.downgrade();
        let slot = runtime.tasks.get_mut(&1).unwrap();
        let domain = slot.worker.domain.clone();
        slot.worker.reset_registers(1);
        slot.worker.store_register(0, Value::Cell(cell)).unwrap();
        slot.worker.publish_execution(&[]).unwrap();
        assert!(weak.upgrade().is_some());
        if shutdown_sleep {
            slot.state = DebugTaskState::Sleeping;
            runtime.cancel_sleeping_tasks();
        } else {
            runtime.cancel_after_root_completion();
        }
        let slot = runtime.tasks.get(&1).unwrap();
        assert_eq!(slot.state, DebugTaskState::Cancelled);
        assert!(slot.worker.execution.is_none());
        assert!(slot.worker.registers.is_empty());
        assert!(
            weak.upgrade().is_none(),
            "terminal peers release frames and unused cases"
        );
        assert_eq!(domain.phase(), fpas_bytecode::DomainPhase::Retired);
    }
}
