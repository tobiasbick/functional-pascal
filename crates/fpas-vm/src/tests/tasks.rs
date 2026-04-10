use crate::Vm;
use fpas_bytecode::{Chunk, Intrinsic, Op, Value};
use fpas_diagnostics::codes::{
    RUNTIME_INVALID_TASK, RUNTIME_VM_OPERAND_TYPE_MISMATCH, RUNTIME_WRONG_CALL_ARITY,
};

use super::helpers::{
    build_function_chunk, build_zero_arg_function_chunk, emit_constant, loc, run_err,
};

#[test]
fn wait_all_with_non_task_value_reports_operand_type_mismatch() {
    let mut chunk = Chunk::new();
    emit_constant(&mut chunk, Value::Integer(1));
    chunk.emit(Op::MakeArray(1), loc());
    chunk.emit(Op::Intrinsic(Intrinsic::TaskWaitAll as u16), loc());
    chunk.emit(Op::Halt, loc());

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
}

#[test]
fn spawn_task_with_wrong_arity_reports_runtime_error() {
    let function_name = "NeedOneArg";
    let chunk = build_function_chunk(
        function_name,
        1,
        |chunk| {
            emit_constant(
                chunk,
                Value::Function {
                    name: function_name.to_string(),
                    captures: vec![],
                },
            );
            chunk.emit(Op::SpawnTask(0), loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(0));
            chunk.emit(Op::Return, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_WRONG_CALL_ARITY);
}

#[test]
fn waiting_twice_on_same_task_reports_runtime_error() {
    let function_name = "ReturnSeven";
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
            chunk.emit(Op::SpawnTask(0), loc());
            chunk.emit(Op::Dup, loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
            chunk.emit(Op::Pop, loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(7));
            chunk.emit(Op::Return, loc());
        },
    );

    let err = run_err(chunk);
    assert_eq!(err.code, RUNTIME_INVALID_TASK);
}

#[test]
fn wait_all_keeps_task_result_available_for_wait() {
    let function_name = "ReturnSeven";
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
            chunk.emit(Op::SpawnTask(0), loc());
            chunk.emit(Op::Dup, loc());
            chunk.emit(Op::MakeArray(1), loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWaitAll as u16), loc());
            chunk.emit(Op::Intrinsic(Intrinsic::TaskWait as u16), loc());
            chunk.emit(Op::PrintLn, loc());
        },
        |chunk| {
            emit_constant(chunk, Value::Integer(7));
            chunk.emit(Op::Return, loc());
        },
    );

    let mut vm = Vm::new(chunk);
    vm.run().expect("WaitAll followed by Wait should succeed");
    assert_eq!(vm.output().lines, vec!["7"]);
}
