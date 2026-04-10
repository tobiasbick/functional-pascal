use crate::Vm;
use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE;

use super::helpers::{emit_constant, loc, run_err};

#[test]
fn malformed_call_reports_error_instead_of_panicking() {
    let mut chunk = Chunk::new();
    let name_idx = chunk
        .add_constant(Value::Str("NeedArg".to_string()))
        .expect("constant should fit in test chunk");
    chunk.functions.insert("NeedArg".to_string(), (1, 1));
    chunk.emit(Op::Call(name_idx, 1), loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    assert!(vm.run().is_err(), "malformed call should return a VM error");
}

#[test]
fn malformed_make_array_reports_error_instead_of_panicking() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::MakeArray(1), loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    assert!(
        vm.run().is_err(),
        "malformed MakeArray should return a VM error"
    );
}

#[test]
fn malformed_get_enclosing_reports_error_instead_of_silently_falling_back() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::GetEnclosing(2, 0), loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    assert!(
        vm.run().is_err(),
        "malformed GetEnclosing should return a VM error"
    );
}

#[test]
fn malformed_field_set_missing_field_reports_error() {
    let mut chunk = Chunk::new();
    let type_idx = chunk
        .add_constant(Value::Str("Point".to_string()))
        .expect("constant should fit in test chunk");
    emit_constant(&mut chunk, Value::Str("x".to_string()));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::MakeRecord(type_idx, 1), loc());
    emit_constant(&mut chunk, Value::Integer(2));

    let missing_field_idx = chunk
        .add_constant(Value::Str("y".to_string()))
        .expect("constant should fit in test chunk");
    chunk.emit(Op::FieldSet(missing_field_idx), loc());
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    assert!(
        vm.run().is_err(),
        "FieldSet on an unknown field should return a VM error"
    );
}

#[test]
fn jump_past_end_reports_internal_vm_error() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Jump(3), loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, INTERNAL_VM_INVARIANT_FAILURE);
}
