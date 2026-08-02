use std::sync::Arc;

use crate::Vm;
use crate::tests::helpers::{emit_constant, loc};
use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::codes::INTERNAL_VM_INVARIANT_FAILURE;

#[test]
fn image_entry_runs_initialization_and_keeps_vm_output_isolated() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Str(("init".to_string()).into()));
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let first_entry = chunk.len();
    emit_constant(&mut chunk, Value::Str(("first".to_string()).into()));
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Unit, loc());
    chunk.emit(Op::Return, loc());

    let second_entry = chunk.len();
    emit_constant(&mut chunk, Value::Str(("second".to_string()).into()));
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Unit, loc());
    chunk.emit(Op::Return, loc());

    let image = Arc::new(chunk);
    let mut first = Vm::from_image(Arc::clone(&image), first_entry);
    let mut second = Vm::from_image(image, second_entry);

    first.run().expect("first image entry must run");
    second.run().expect("second image entry must run");

    assert_eq!(first.output().lines, vec!["init", "first"]);
    assert_eq!(second.output().lines, vec!["init", "second"]);
}

#[test]
fn image_entry_at_code_length_is_rejected() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Halt, loc());
    let entry = chunk.len();
    let mut vm = Vm::from_image(Arc::new(chunk), entry);

    let error = vm.run().expect_err("entry at code length must fail");
    assert_eq!(error.code, INTERNAL_VM_INVARIANT_FAILURE);
    assert!(error.message.contains("Bytecode entry"));
}

#[test]
fn image_entry_above_code_length_is_rejected() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Halt, loc());
    let entry = chunk.len() + 1;
    let mut vm = Vm::from_image(Arc::new(chunk), entry);

    let error = vm.run().expect_err("entry above code length must fail");
    assert_eq!(error.code, INTERNAL_VM_INVARIANT_FAILURE);
    assert!(error.message.contains("out of bounds"));
}
