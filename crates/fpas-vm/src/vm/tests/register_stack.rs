use fpas_bytecode::{Constant, FunctionId, Opcode, ReturnConvention, Value};

use crate::vm::Vm;

use super::calls::{FunctionSpec, abc, image};
use super::support::abx;

#[test]
fn reused_call_frame_registers_do_not_expose_previous_values() {
    let executable = image(
        vec![
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::CallDirect, fpas_bytecode::NO_REGISTER, 2, 0, 0),
            abc(Opcode::CallDirect, 0, 3, 0, 0),
            abc(Opcode::Return, 0, 0, 0, 0),
            abx(Opcode::LoadConstant, 1, 0),
            abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0),
            abc(Opcode::Return, 1, 0, 0, 0),
        ],
        vec![Constant::Integer(99)],
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
                end: 4,
                arity: 0,
                captures: 0,
                registers: 1,
                returns: ReturnConvention::Value,
            },
            FunctionSpec {
                start: 4,
                end: 6,
                arity: 0,
                captures: 0,
                registers: 2,
                returns: ReturnConvention::Unit,
            },
            FunctionSpec {
                start: 6,
                end: 7,
                arity: 0,
                captures: 0,
                registers: 2,
                returns: ReturnConvention::Value,
            },
        ],
    );

    let result = Vm::new(executable)
        .call(FunctionId::new(1), Vec::new())
        .expect("reused call frame should return successfully");
    assert_eq!(result.value, Value::Unit);
}

#[test]
fn released_and_reused_register_slots_are_explicitly_uninitialized() {
    let executable = super::verified(
        vec![super::return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        4,
    );
    let mut worker =
        crate::vm::worker::Worker::new(std::sync::Arc::new(executable)).expect("worker");
    assert!(
        (0..4).all(|index| !worker.register_is_initialized(index)),
        "fresh slots start uninitialized"
    );

    worker
        .store_register(2, Value::Integer(9))
        .expect("initialize a live slot");
    assert!(worker.register_is_initialized(2));

    worker.release_registers(1);
    assert!(!worker.register_is_initialized(2));
    assert_eq!(
        worker.registers.len(),
        1,
        "released slots leave the active window"
    );
    assert!(
        worker.registers.capacity() >= 4,
        "released storage stays allocated"
    );

    worker.activate_registers(4);
    assert!(
        (1..4).all(|index| !worker.register_is_initialized(index)),
        "reused slots stay uninitialized"
    );
    assert!(
        worker.registers[1..4]
            .iter()
            .all(|value| matches!(value, Value::Unit)),
        "reused slots never expose previous values"
    );
}

#[test]
fn integer_overwrite_preserves_the_slot_and_releases_previous_references() {
    use std::sync::{Arc, Mutex};

    let executable = super::verified(
        vec![super::return_unit()],
        Vec::new(),
        vec!["root", "test.fpas"],
        1,
    );
    let mut worker = crate::vm::worker::Worker::new(Arc::new(executable)).expect("worker");
    let cell = Arc::new(Mutex::new(Value::Unit));
    worker
        .store_register(0, Value::Cell(Arc::clone(&cell)))
        .expect("store reference");
    assert_eq!(Arc::strong_count(&cell), 2);

    worker
        .store_register(0, Value::Integer(3))
        .expect("replace reference");
    assert_eq!(Arc::strong_count(&cell), 1);
    worker
        .store_register(0, Value::Integer(4))
        .expect("replace integer");
    assert_eq!(worker.registers[0], Value::Integer(4));
    assert!(worker.register_is_initialized(0));
}
