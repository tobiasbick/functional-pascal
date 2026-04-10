use crate::Vm;
use fpas_bytecode::{Chunk, Op, SourceLocation, Value};

pub(super) fn loc() -> SourceLocation {
    SourceLocation::new(1, 1)
}

pub(super) fn emit_constant(chunk: &mut Chunk, value: Value) {
    let idx = chunk
        .add_constant(value)
        .expect("constant should fit in test chunk");
    chunk.emit(Op::Constant(idx), loc());
}

pub(super) fn build_function_chunk(
    function_name: &str,
    arity: u8,
    main: impl FnOnce(&mut Chunk),
    body: impl FnOnce(&mut Chunk),
) -> Chunk {
    let mut chunk = Chunk::new();
    main(&mut chunk);
    chunk.emit(Op::Halt, loc());

    let code_start = chunk.len();
    chunk
        .functions
        .insert(function_name.to_string(), (code_start, arity));
    body(&mut chunk);
    chunk
}

pub(super) fn build_zero_arg_function_chunk(
    function_name: &str,
    main: impl FnOnce(&mut Chunk),
    body: impl FnOnce(&mut Chunk),
) -> Chunk {
    build_function_chunk(function_name, 0, main, body)
}

pub(super) fn run_err(chunk: Chunk) -> fpas_diagnostics::Diagnostic {
    let mut vm = Vm::new(chunk);
    vm.run().expect_err("VM should return an error")
}

pub(super) fn run_ok_output(chunk: Chunk) -> Vec<String> {
    let mut vm = Vm::new(chunk);
    vm.run().expect("VM should succeed");
    vm.output().lines
}
