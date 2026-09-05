use super::*;

#[test]
fn array_pop_reuses_unique_storage_and_keeps_invalid_sources() {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeArray, 1, 0, 1),
            abc(Opcode::ArrayPop, 2, 1, 0),
            abc(Opcode::ArrayPop, 2, 1, 0),
            return_unit(),
        ],
        vec![Constant::Integer(7)],
        vec!["root", "test.fpas"],
        3,
    );
    let mut worker = crate::vm::worker::Worker::new(Arc::new(executable)).expect("worker");
    worker.dispatch_one().expect("constant");
    worker.dispatch_one().expect("array");
    let Value::Array(array) = &worker.registers[1] else {
        panic!("array");
    };
    let storage = array.as_ptr();
    worker.dispatch_one().expect("pop");
    let Value::Array(array) = &worker.registers[1] else {
        panic!("array");
    };
    assert_eq!(array.as_ptr(), storage);
    assert!(array.is_empty());
    assert_eq!(worker.registers[2], Value::Integer(7));
    let error = worker.dispatch_one().err().expect("empty pop");
    assert_eq!(error.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);
    assert!(worker.register_is_initialized(1));
    assert_eq!(worker.registers[1], Value::Array(Vec::new().into()));
    assert_eq!(worker.registers[2], Value::Integer(7));
}

#[test]
fn array_pop_rejects_non_array_before_consuming_it() {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::ArrayPop, 1, 0, 0),
            return_unit(),
        ],
        vec![Constant::Integer(7)],
        vec!["root", "test.fpas"],
        2,
    );
    let mut worker = crate::vm::worker::Worker::new(Arc::new(executable)).expect("worker");
    worker.dispatch_one().expect("constant");
    assert_eq!(
        worker.dispatch_one().err().expect("non-array").code,
        RUNTIME_VM_OPERAND_TYPE_MISMATCH
    );
    assert_eq!(worker.registers[0], Value::Integer(7));
    assert!(worker.register_is_initialized(0));
}
