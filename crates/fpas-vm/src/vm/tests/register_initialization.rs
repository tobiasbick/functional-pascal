//! Explicit register initialization bits across writes, calls, callbacks, and tasks.

use std::sync::Arc;

use fpas_bytecode::{
    ArgsIntrinsic, FunctionId, InstructionAddress, Intrinsic, Opcode, Register, ReturnConvention,
    Value,
};

use super::calls::{FunctionSpec, abc, image};
use super::support::{abx, verified};
use crate::vm::debug::initializer_suppression::SourceInitializerTarget;
use crate::vm::dispatch::DispatchStep;
use crate::vm::worker::Worker;

#[test]
fn normal_dispatch_ignores_debug_initializer_suppression() {
    let executable = verified(
        vec![abc(Opcode::LoadUnit, 0, 0, 0, 0), super::return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        1,
    );
    let mut worker = Worker::new(Arc::new(executable)).expect("worker");
    worker.suppress_source_initializer(SourceInitializerTarget {
        function: FunctionId::new(0),
        base: 0,
        instruction: InstructionAddress::new(0),
    });

    assert!(matches!(
        worker.dispatch_one().expect("normal dispatch"),
        DispatchStep::Continue
    ));
    assert!(worker.register_is_initialized(0));
}

#[test]
fn debug_dispatch_applies_initializer_suppression() {
    let executable = verified(
        vec![abc(Opcode::LoadUnit, 0, 0, 0, 0), super::return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        1,
    );
    let mut worker = Worker::new(Arc::new(executable)).expect("worker");
    worker.suppress_source_initializer(SourceInitializerTarget {
        function: FunctionId::new(0),
        base: 0,
        instruction: InstructionAddress::new(0),
    });

    assert!(matches!(
        worker.dispatch_debug_one().expect("debug dispatch"),
        DispatchStep::Continue
    ));
    assert!(!worker.register_is_initialized(0));
}

#[test]
fn writes_takes_and_unit_stores_update_initialization_bits() {
    let executable = verified(
        vec![super::return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        3,
    );
    let mut worker = Worker::new(Arc::new(executable)).expect("worker");
    let slot = Register::new(1).expect("register");

    worker.write(slot, Value::Unit).expect("initialized unit");
    assert!(worker.register_is_initialized(1));
    assert_eq!(worker.registers[1], Value::Unit);

    let taken = worker.take(slot).expect("take clears the source");
    assert_eq!(taken, Value::Unit);
    assert!(!worker.register_is_initialized(1));
    assert_eq!(worker.registers[1], Value::Unit);
}

#[test]
fn call_arguments_captures_and_return_destinations_are_initialized() {
    let executable = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::CallDirect, 2, 2, 0, 2),
            abc(Opcode::Return, 2, 0, 0, 0),
            abc(Opcode::AddInteger, 2, 0, 1, 0),
            abc(Opcode::Return, 2, 0, 0, 0),
        ],
        vec![
            fpas_bytecode::Constant::Integer(20),
            fpas_bytecode::Constant::Integer(22),
        ],
        &[
            FunctionSpec {
                start: 0,
                end: 1,
                arity: 0,
                captures: 0,
                registers: 0,
                returns: ReturnConvention::Unit,
            },
            FunctionSpec {
                start: 1,
                end: 5,
                arity: 0,
                captures: 0,
                registers: 3,
                returns: ReturnConvention::Value,
            },
            FunctionSpec {
                start: 5,
                end: 7,
                arity: 2,
                captures: 0,
                registers: 4,
                returns: ReturnConvention::Value,
            },
        ],
    );
    let mut worker = Worker::new(Arc::new(executable)).expect("worker");
    worker.function = FunctionId::new(1);
    worker.ip = 1;
    worker.reset_registers(3);
    loop {
        match worker.dispatch_one().expect("dispatch") {
            DispatchStep::Continue => {
                if worker.function == FunctionId::new(2) {
                    assert!(worker.register_is_initialized(worker.base));
                    assert!(worker.register_is_initialized(worker.base + 1));
                    assert!(!worker.register_is_initialized(worker.base + 3));
                }
            }
            DispatchStep::Suspend => panic!("unexpected suspend"),
            DispatchStep::Return(_) => break,
        }
    }
    assert!(worker.register_is_initialized(2));
    assert!(!worker.register_is_initialized(3));
}

#[test]
fn callback_and_debug_intrinsic_argument_windows_are_initialized() {
    let executable = verified(
        vec![super::return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        1,
    );
    let mut worker = Worker::new(Arc::new(executable)).expect("worker");
    worker
        .execute_debug_intrinsic(Intrinsic::Args(ArgsIntrinsic::ParamCount), &[])
        .expect("debug intrinsic");
    assert!(worker.register_is_initialized(0));

    let callback_image = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::AddInteger, 1, 0, 0, 0),
            abc(Opcode::Return, 1, 0, 0, 0),
        ],
        Vec::new(),
        &[
            FunctionSpec {
                start: 0,
                end: 1,
                arity: 0,
                captures: 0,
                registers: 0,
                returns: ReturnConvention::Unit,
            },
            FunctionSpec {
                start: 1,
                end: 3,
                arity: 1,
                captures: 0,
                registers: 3,
                returns: ReturnConvention::Value,
            },
        ],
    );
    let callback = Worker::new(Arc::new(callback_image)).expect("callback worker");
    let function = Value::function(FunctionId::new(1), "double".to_string(), Vec::new());
    assert_eq!(
        callback
            .call_callback_sync(&function, &[Value::Integer(3)])
            .expect("callback"),
        Value::Integer(6)
    );
    let stored = callback.callback_worker.borrow();
    let stored = stored.as_ref().expect("reused callback worker");
    assert!(stored.register_is_initialized(0));
    assert!(stored.register_is_initialized(1));
    assert!(!stored.register_is_initialized(2));
}

#[test]
fn task_yield_and_resume_preserve_per_register_initialization() {
    let executable = verified(
        vec![super::return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        3,
    );
    let mut worker = Worker::new(Arc::new(executable)).expect("worker");
    worker
        .store_register(0, Value::Integer(4))
        .expect("initialized slot");
    worker
        .write(Register::new(1).expect("register"), Value::Unit)
        .expect("initialized unit");

    let state = worker.take_task_state();
    assert_eq!(state.register_initialized, vec![true, true, false]);
    let restored = worker.worker_for_task(state);
    assert!(restored.register_is_initialized(0));
    assert!(restored.register_is_initialized(1));
    assert!(!restored.register_is_initialized(2));
    assert_eq!(restored.registers[1], Value::Unit);
}
