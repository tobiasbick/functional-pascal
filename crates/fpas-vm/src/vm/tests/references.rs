//! Capture-cell opcodes obey the shared storage-root access registry.

use std::sync::{Arc, Mutex};

use fpas_bytecode::{
    AbcOperands, CellBorrow, CodeRange, Constant, Instruction, InstructionAddress, NO_REGISTER,
    Opcode, Value,
};
use fpas_diagnostics::codes::RUNTIME_STORAGE_REFERENCE_CONFLICT;

use crate::vm::worker::Worker;

use super::{abc, abc_aux, abx, return_unit};

mod escape;
mod fixture;
mod invocation;
mod lifetime;
mod paths;

fn worker() -> Worker {
    let executable = super::verified(
        vec![super::return_unit()],
        Vec::new(),
        vec!["root", "reference-access.fpas"],
        3,
    );
    Worker::new(Arc::new(executable)).unwrap()
}

fn reserve(worker: &mut Worker) -> CellBorrow {
    let root = Arc::new(Mutex::new(Value::Integer(1)));
    worker
        .store_register(0, Value::Cell(Arc::clone(&root)))
        .unwrap();
    worker.store_register(2, Value::Integer(2)).unwrap();
    worker.hosted.references.reserve(&root).unwrap()
}

fn operands(a: u16, b: u16) -> AbcOperands {
    AbcOperands {
        a,
        b,
        c: 0,
        auxiliary: 0,
    }
}

#[test]
fn capture_cell_read_allows_reserved_snapshot_and_rejects_exclusive_alias() {
    let mut worker = worker();
    let borrowed = reserve(&mut worker);
    worker.read_cell(operands(1, 0)).unwrap();
    assert_eq!(worker.registers[1], Value::Integer(1));
    borrowed.activate().unwrap();
    let error = worker.read_cell(operands(1, 0)).unwrap_err();
    assert_eq!(error.code, RUNTIME_STORAGE_REFERENCE_CONFLICT);
    assert!(error.message.contains("exclusively"));
    assert!(error.span.is_some());
    assert!(
        error
            .help
            .as_deref()
            .is_some_and(|help| help.contains("var parameter"))
    );
}

#[test]
fn capture_cell_write_rejects_reserved_alias_without_replacing_storage() {
    let mut worker = worker();
    let borrowed = reserve(&mut worker);
    let error = worker.write_cell(operands(0, 2)).unwrap_err();
    assert_eq!(error.code, RUNTIME_STORAGE_REFERENCE_CONFLICT);
    assert_eq!(borrowed.read().unwrap(), Value::Integer(1));
    borrowed.activate().unwrap();
    borrowed.write(Value::Integer(5)).unwrap();
    borrowed.release().unwrap();
    worker.read_cell(operands(1, 0)).unwrap();
    assert_eq!(worker.registers[1], Value::Integer(5));
}

#[test]
fn another_worker_shares_alias_exclusion_with_its_callback_host() {
    let mut parent = worker();
    let borrowed = reserve(&mut parent);
    borrowed.activate().unwrap();
    let mut callback = Worker::for_function_with_state(
        Arc::clone(&parent.executable),
        parent.function,
        Vec::new(),
        Arc::clone(&parent.globals),
        Arc::clone(&parent.layouts),
        Arc::clone(&parent.hosted),
    )
    .unwrap();
    callback
        .store_register(0, parent.registers[0].clone())
        .unwrap();
    callback.store_register(2, Value::Integer(9)).unwrap();
    let error = callback.write_cell(operands(0, 2)).unwrap_err();
    assert_eq!(error.code, RUNTIME_STORAGE_REFERENCE_CONFLICT);
    borrowed.release().unwrap();
    callback.write_cell(operands(0, 2)).unwrap();
    parent.read_cell(operands(1, 0)).unwrap();
    assert_eq!(parent.registers[1], Value::Integer(9));
}
