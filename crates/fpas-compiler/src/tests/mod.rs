use fpas_parser::parse;

fn parse_ok(source: &str) -> fpas_parser::Program {
    let (program, errors) = parse(source);
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    program
}

fn run_program(source: &str) -> Result<fpas_vm::Execution, fpas_vm::VmError> {
    let program = parse_ok(source);
    let executable = crate::compile(&program).expect("compilation should succeed");
    fpas_vm::Vm::new(executable).run()
}

fn assert_succeeds(source: &str) -> fpas_vm::Execution {
    run_program(source).expect("program should succeed")
}

mod aggregates;
mod closures;
mod concurrency;
mod control_flow;
mod debug;
mod diagnostics;
mod functions;
mod intrinsics;
mod structure;
