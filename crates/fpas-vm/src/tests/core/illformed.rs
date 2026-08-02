use crate::Vm;
use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE;

use crate::tests::helpers::{emit_constant, loc, run_err};

#[test]
fn malformed_call_reports_error_instead_of_panicking() {
    let mut chunk = Chunk::new();
    let name_idx = chunk
        .add_constant(Value::Str(("NeedArg".to_string()).into()))
        .expect("constant should fit in test chunk");
    chunk.insert_function("NeedArg".to_string(), 1, 1);
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
        .add_constant(Value::Str(("Point".to_string()).into()))
        .expect("constant should fit in test chunk");
    emit_constant(&mut chunk, Value::Str(("x".to_string()).into()));
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::MakeRecord(type_idx, 1), loc());
    emit_constant(&mut chunk, Value::Integer(2));

    let missing_field_idx = chunk
        .add_constant(Value::Str(("y".to_string()).into()))
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

#[test]
fn main_code_boundary_without_halt_reports_internal_vm_error() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Unit, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, INTERNAL_VM_INVARIANT_FAILURE);
    assert!(err.message.contains("code boundary"));
    assert!(err.message.contains("context=main task"));
}

#[test]
fn call_target_at_code_length_is_rejected_before_entering_frame() {
    let mut chunk = Chunk::new();
    let name_idx = chunk
        .add_constant(Value::Str(("MissingBody".to_string()).into()))
        .expect("constant should fit in test chunk");
    chunk.emit(Op::Call(name_idx, 0), loc());
    chunk.emit(Op::Halt, loc());
    chunk.insert_function("MissingBody".to_string(), chunk.len(), 0);

    let err = run_err(chunk);
    assert_eq!(err.code, INTERNAL_VM_INVARIANT_FAILURE);
    assert!(err.message.contains("Bytecode entry"));
    assert!(err.message.contains("out of bounds"));
}

#[test]
fn code_boundary_with_active_call_frame_is_rejected() {
    let mut chunk = Chunk::new();
    let name_idx = chunk
        .add_constant(Value::Str(("FallsOff".to_string()).into()))
        .expect("constant should fit in test chunk");
    chunk.emit(Op::Call(name_idx, 0), loc());
    chunk.emit(Op::Halt, loc());
    let function_start = chunk.len();
    chunk.insert_function("FallsOff".to_string(), function_start, 0);
    chunk.emit(Op::Unit, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, INTERNAL_VM_INVARIANT_FAILURE);
    assert!(err.message.contains("code boundary"));
}

#[test]
fn halt_inside_main_call_frame_is_rejected() {
    let mut chunk = Chunk::new();
    let name_idx = chunk
        .add_constant(Value::Str(("Halts".to_string()).into()))
        .expect("constant should fit in test chunk");
    chunk.emit(Op::Call(name_idx, 0), loc());
    chunk.emit(Op::Halt, loc());
    let function_start = chunk.len();
    chunk.insert_function("Halts".to_string(), function_start, 0);
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, INTERNAL_VM_INVARIANT_FAILURE);
    assert!(err.message.contains("Halt is invalid"));
}

#[test]
fn malformed_add_int_reports_internal_vm_error() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::AddInt, loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, INTERNAL_VM_INVARIANT_FAILURE);
}
