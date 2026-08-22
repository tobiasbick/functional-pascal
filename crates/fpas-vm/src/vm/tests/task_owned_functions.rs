//! Runtime ownership regressions for first-class functions with mutable captures.

use std::sync::Arc;

use fpas_bytecode::{Constant, FunctionId, Opcode, ReturnConvention, Value};
use fpas_diagnostics::codes::RUNTIME_INVALID_TASK;

use super::calls::{FunctionSpec, abc, image};
use super::support::{abx, execute};
use crate::vm::dispatch::DispatchStep;
use crate::vm::worker::Worker;

#[test]
fn mutable_cell_capture_marks_numeric_closure_task_bound() {
    let executable = image(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeCell, 0, 0, 0, 0),
            abc(Opcode::MakeClosure, 1, 1, 0, 1),
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
        ],
        vec![Constant::Integer(1)],
        &[
            FunctionSpec {
                start: 0,
                end: 4,
                arity: 0,
                captures: 0,
                registers: 2,
                returns: ReturnConvention::Unit,
            },
            FunctionSpec {
                start: 4,
                end: 5,
                arity: 0,
                captures: 1,
                registers: 1,
                returns: ReturnConvention::Unit,
            },
        ],
    );

    let (_, registers, _) = execute(executable).expect("closure construction should succeed");
    let Value::Function(function) = &registers[1] else {
        panic!("register must contain closure")
    };
    assert!(function.task_bound);
    assert_eq!(function.owner_task, Some(0));
    assert_eq!(function.function, FunctionId::new(1));
}

#[test]
fn task_owned_closure_rejects_call_value_from_a_foreign_task() {
    let executable = image(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeCell, 0, 0, 0, 0),
            abc(Opcode::MakeClosure, 1, 1, 0, 1),
            abc(Opcode::CallValue, 2, 1, 0, 0),
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
        ],
        vec![Constant::Integer(1)],
        &[
            FunctionSpec {
                start: 0,
                end: 5,
                arity: 0,
                captures: 0,
                registers: 3,
                returns: ReturnConvention::Unit,
            },
            FunctionSpec {
                start: 5,
                end: 6,
                arity: 0,
                captures: 1,
                registers: 1,
                returns: ReturnConvention::Unit,
            },
        ],
    );
    let mut worker = Worker::new(Arc::new(executable)).expect("worker");
    for _ in 0..3 {
        match worker.dispatch_one().expect("setup") {
            DispatchStep::Continue => {}
            DispatchStep::Suspend | DispatchStep::Return(_) => panic!("unexpected setup step"),
        }
    }
    let Value::Function(function) = &worker.registers[1] else {
        panic!("closure")
    };
    assert_eq!(function.owner_task, Some(0));

    worker.task_id = 5;
    let error = worker
        .dispatch_one()
        .err()
        .expect("a foreign task must not invoke the closure");

    assert_eq!(error.code, RUNTIME_INVALID_TASK);
    assert!(error.message.contains("foreign task"), "{}", error.message);
}
