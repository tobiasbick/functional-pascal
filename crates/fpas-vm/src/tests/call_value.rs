use fpas_bytecode::{Chunk, Op, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;

use super::helpers::{build_zero_arg_function_chunk, emit_constant, loc, run_err, run_ok_output};

#[test]
fn call_value_executes_function_value_and_returns_result() {
    let function_name = "ReturnNine";
    let chunk = build_zero_arg_function_chunk(
        function_name,
        |chunk| {
            emit_constant(
                chunk,
                Value::Function {
                    name: function_name.to_string(),
                    captures: vec![],
                },
            );
            chunk.emit(Op::CallValue(0), loc());
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(9));
            chunk.emit(Op::Return, loc());
        },
    );

    assert_eq!(run_ok_output(chunk), vec!["9"]);
}

#[test]
fn call_value_resolves_function_name_case_insensitively() {
    let mut chunk = Chunk::new();
    emit_constant(
        &mut chunk,
        Value::Function {
            name: "ReturnNine".to_string(),
            captures: vec![],
        },
    );
    chunk.emit(Op::CallValue(0), loc());
    chunk.emit(Op::PrintLn, loc());
    chunk.emit(Op::Halt, loc());

    let code_start = chunk.len();
    chunk
        .functions
        .insert("returnnine".to_string(), (code_start, 0));
    emit_constant(&mut chunk, Value::Integer(9));
    chunk.emit(Op::Return, loc());

    assert_eq!(run_ok_output(chunk), vec!["9"]);
}

#[test]
fn call_value_with_non_function_reports_operand_type_mismatch() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::CallValue(0), loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
}
