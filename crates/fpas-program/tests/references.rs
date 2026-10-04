//! Synchronous parameter modes and their portable types survive program encoding.

mod common;

#[test]
fn reference_modes_and_type_graph_round_trip_without_sources() {
    let original = common::program_image();
    let mut executable = original.executable().executable().clone();
    executable.functions[1].arity = 1;
    executable.functions[1].capture_count = 0;
    executable.functions[1].var_parameters = vec![0];
    executable.functions[1].return_convention = fpas_bytecode::ReturnConvention::Unit;
    executable.functions[1].debug = fpas_bytecode::FunctionDebugInfo::default();
    executable.code[4] = fpas_bytecode::Instruction::abc(
        fpas_bytecode::Opcode::Return,
        fpas_bytecode::NO_REGISTER,
        0,
        0,
        0,
    )
    .unwrap();
    executable
        .debug_types
        .push(fpas_bytecode::DebugType::Reference(
            fpas_bytecode::DebugTypeId::new(2),
        ));
    let executable = executable.verify().unwrap();
    let image = fpas_program::ProgramImage::new(
        original.identity().clone(),
        original.source_paths().to_vec(),
        original.source_hashes().to_vec(),
        executable,
    )
    .unwrap();
    let bytes = fpas_program::encode(&image).unwrap();
    let decoded = fpas_program::decode(&bytes).unwrap();
    assert_eq!(
        decoded.executable().executable().functions[1].var_parameters,
        vec![0]
    );
    assert_eq!(
        decoded.executable().executable().debug_types,
        image.executable().executable().debug_types
    );
}
