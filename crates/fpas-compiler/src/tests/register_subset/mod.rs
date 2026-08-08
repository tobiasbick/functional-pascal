use fpas_parser::parse;

fn parse_ok(source: &str) -> fpas_parser::Program {
    let (program, errors) = parse(source);
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    program
}

fn run_old(source: &str) -> Result<(), fpas_vm::VmError> {
    let program = parse_ok(source);
    let chunk = crate::compile(&program).expect("stack compilation should succeed");
    fpas_vm::Vm::new(chunk).run()
}

fn run_register(source: &str) -> Result<fpas_vm::RegisterExecution, fpas_vm::VmError> {
    let program = parse_ok(source);
    let executable = crate::compile_register_subset(&program)
        .expect("register subset compilation should succeed");
    fpas_vm::RegisterVm::new(executable).run()
}

fn assert_both_succeed(source: &str) -> fpas_vm::RegisterExecution {
    run_old(source).expect("stack path should succeed");
    run_register(source).expect("register path should succeed")
}

mod aggregates;
mod closures;
mod control_flow;
mod diagnostics;
mod functions;
mod intrinsics;
mod structure;
