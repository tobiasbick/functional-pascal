use fpas_parser::parse;

fn parse_ok(source: &str) -> fpas_parser::Program {
    let (program, errors) = parse(source);
    assert!(errors.is_empty(), "parse errors: {errors:?}");
    program
}

fn compiled_unit(
    source: &str,
    interfaces: &[fpas_unit::interface::UnitInterface],
) -> crate::CompiledUnitObject {
    let (parsed, errors) = fpas_parser::parse_compilation_unit(source);
    assert!(errors.is_empty(), "{errors:?}");
    let fpas_parser::CompilationUnit::Unit(unit) = parsed else {
        panic!("unit expected");
    };
    let mut compiled = crate::compile_unit_object(&unit, interfaces).expect("compiled unit");
    compiled.interface = fpas_unit::interface::decode_interface(
        &fpas_unit::interface::encode_interface(&compiled.interface).expect("encode"),
    )
    .expect("decode");
    compiled
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
mod call_frames;
mod canonical_collections;
mod closures;
mod computed_const;
mod concurrency;
mod contracts;
mod control_flow;
mod debug;
mod diagnostics;
mod discard;
mod error_conventions;
mod functions;
mod intrinsics;
mod keyword_switch;
mod local_inference;
mod numeric_contracts;
mod optimization;
mod structure;
mod subranges;
