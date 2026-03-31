use fpas_bytecode::Op;
use fpas_parser::parse;
use fpas_std::{ConsoleKeyEvent, key_event::key_kind_index};

fn compile_ok(source: &str) -> fpas_bytecode::Chunk {
    let (program, errors) = parse(source);
    assert!(errors.is_empty(), "Parse errors: {errors:?}");
    crate::compile(&program).expect("Compilation should succeed")
}

fn compile_and_run(source: &str) -> fpas_vm::VmOutput {
    let chunk = compile_ok(source);
    let mut vm = fpas_vm::Vm::new(chunk);
    vm.run().expect("VM should not error");
    vm.output().clone()
}

fn compile_run_with_readln(source: &str, inputs: &[&str]) -> fpas_vm::VmOutput {
    let chunk = compile_ok(source);
    let mut vm = fpas_vm::Vm::new(chunk);
    for line in inputs {
        vm.push_readln_input(line);
    }
    vm.run().expect("VM should not error");
    vm.output().clone()
}

fn compile_run_with_readln_and_readkey(
    source: &str,
    lines: &[&str],
    keys: &str,
) -> fpas_vm::VmOutput {
    let chunk = compile_ok(source);
    let mut vm = fpas_vm::Vm::new(chunk);
    for line in lines {
        vm.push_readln_input(line);
    }
    vm.push_readkey_input(keys);
    vm.run().expect("VM should not error");
    vm.output().clone()
}

fn compile_run_err(source: &str) -> String {
    compile_run_error(source).message
}

fn compile_run_error(source: &str) -> fpas_vm::VmError {
    let chunk = compile_ok(source);
    let mut vm = fpas_vm::Vm::new(chunk);
    match vm.run() {
        Ok(()) => panic!("expected VM runtime error"),
        Err(e) => e,
    }
}

fn compile_err(source: &str) -> crate::CompileError {
    let (program, errors) = parse(source);
    assert!(errors.is_empty(), "Parse errors: {errors:?}");
    crate::compile(&program).expect_err("compilation should fail")
}

fn parse_fails(source: &str) {
    let (_, errors) = parse(source);
    assert!(!errors.is_empty(), "Expected parse errors, got none");
}

mod arrays;
mod basics;
mod case_of;
mod case_ranges;
mod char_type;
mod closures;
mod control_flow;
mod diagnostics;
mod enums;
mod expressions;
mod functions;
mod functions_errors;
mod generics;
mod nested_functions;
mod numeric_binary_ops;
mod pattern_matching;
mod records;
mod result_option;
mod routine_declarations;
mod short_names;
mod std_library;
mod type_aliases;
mod writeln_semantics;
