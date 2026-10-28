//! Task admission rejects complete local graphs before publishing task ownership.

use std::sync::Arc;

use crate::{FunctionId, Instruction, Opcode, SharedFunction, Value};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

use crate::execution::vm::TaskResultPoll;
use crate::execution::vm::tasks::{TaskScheduler, pool};
use crate::execution::vm::worker::Worker;

fn worker() -> (Worker, SharedFunction) {
    let (program, errors) = fpas_parser::parse(
        "program Transfers; function Work(Item: integer): integer; begin return Item; end function; begin end.",
    );
    assert!(errors.is_empty(), "{errors:?}");
    let image = Arc::new(fpas_compiler::compile(&program).expect("fixture"));
    let index = image
        .executable()
        .functions
        .iter()
        .position(|info| image.executable().strings.get(info.name) == Some("work"))
        .expect("worker function");
    let function = SharedFunction::unbound(
        image
            .function_target(FunctionId::try_from_index(index).expect("id"))
            .expect("target"),
        vec![],
    );
    let mut worker = Worker::new(image)
        .expect("worker")
        .with_scheduler(Some(Arc::new(TaskScheduler::new())));
    worker.registers = vec![Value::Unit; 3];
    worker.register_initialized = vec![true; 3];
    worker.enter_task_scope().expect("scope");
    (worker, function)
}

fn spawn(
    worker: &mut Worker,
    function: SharedFunction,
    argument: Value,
) -> Result<(), crate::execution::vm::VmError> {
    worker.registers[1] = Value::Function(function);
    worker.registers[2] = argument;
    worker.spawn_task(
        Instruction::abc(Opcode::SpawnTask, 0, 1, 2, 1)
            .expect("opcode")
            .abc_payload(),
    )
}

#[test]
fn direct_and_nested_arguments_reject_before_task_id_result_or_scope_admission() {
    for wrapped in [false, true] {
        let (mut worker, function) = worker();
        let cell = Value::Cell(worker.domain.new_cell(Value::Integer(7)).expect("cell"));
        let argument = if wrapped {
            Value::option_some(Value::Array(vec![cell].into()))
        } else {
            cell
        };
        let error = spawn(&mut worker, function, argument).expect_err("local argument");
        assert_eq!(error.code, RUNTIME_INVALID_TASK);
        assert_eq!(worker.registers[0], Value::Unit);
        let scheduler = worker.scheduler_ref().expect("scheduler");
        assert!(scheduler.try_dequeue().is_none());
        assert!(matches!(scheduler.poll_result(1), TaskResultPoll::Unknown));
        assert_eq!(
            scheduler.alloc_id(),
            1,
            "rejection must not allocate a task"
        );
        worker
            .exit_task_scope(false)
            .expect("empty scope completes");
    }
}

#[test]
fn unbound_callable_cannot_hide_local_state_in_captures_or_bound_receiver() {
    for receiver in [false, true] {
        let (mut worker, function) = worker();
        let local = Value::option_some(Value::Cell(
            worker.domain.new_cell(Value::Unit).expect("cell"),
        ));
        let function = if receiver {
            SharedFunction::bound(function.target().clone(), local)
        } else {
            SharedFunction::unbound(function.target().clone(), vec![local])
        };
        worker.registers[1] = Value::Function(function);
        worker.registers[2] = Value::Integer(4);
        let count = u8::from(!receiver);
        let error = worker
            .spawn_task(
                Instruction::abc(Opcode::SpawnTask, 0, 1, 2, count)
                    .expect("opcode")
                    .abc_payload(),
            )
            .expect_err("hidden local graph");
        assert_eq!(error.code, RUNTIME_INVALID_TASK);
        assert!(
            worker
                .scheduler_ref()
                .expect("scheduler")
                .try_dequeue()
                .is_none()
        );
        worker
            .exit_task_scope(false)
            .expect("empty scope completes");
    }
}

#[test]
fn immutable_argument_still_completes_and_releases_scope_accounting() {
    let (mut worker, function) = worker();
    spawn(&mut worker, function, Value::Integer(42)).expect("transferable start");
    assert_eq!(worker.registers[0], Value::Task(1));
    let scheduler = Arc::clone(worker.scheduler_ref().expect("scheduler"));
    let task = scheduler.try_dequeue().expect("admitted task");
    assert_ne!(task.domain.id(), worker.domain.id());
    pool::run_helped(&worker, task, Arc::clone(&scheduler)).expect("task completion");
    assert!(matches!(
        scheduler.poll_result(1),
        TaskResultPoll::Available(Value::Integer(42))
    ));
    worker.exit_task_scope(false).expect("completed scope");
}
