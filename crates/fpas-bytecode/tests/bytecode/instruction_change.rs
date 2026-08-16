//! Evidence that verification does not prove interior instruction-change safety.

use fpas_bytecode::{
    DebugScope, DebugSourceLocation, FunctionDebugInfo, InstructionAddress, Opcode, SequencePoint,
    SourceId,
};

use super::support::{
    abx, all_opcodes_executable, minimal_executable, replace_root_code, return_unit,
};

fn point(instruction: u32, line: u32) -> SequencePoint {
    SequencePoint {
        instruction: InstructionAddress::new(instruction),
        location: DebugSourceLocation {
            source: SourceId::new(0),
            line,
            column: 1,
        },
        scope: 0,
    }
}

#[test]
fn sequence_points_verify_without_per_instruction_dataflow() {
    let mut executable = minimal_executable();
    replace_root_code(
        &mut executable,
        vec![abx(Opcode::Jump, 0, 2), return_unit(), return_unit()],
    );
    executable.functions[0].debug = FunctionDebugInfo {
        scopes: vec![DebugScope {
            id: 0,
            parent: None,
        }],
        bindings: Vec::new(),
        sequence_points: vec![point(0, 1), point(2, 2)],
        result_type: None,
        lexical_owner: None,
        capture_sources: Vec::new(),
    };
    executable.source_map.runs[0].instruction_start = InstructionAddress::new(0);

    let verified = executable
        .verify()
        .expect("interior sequence points are valid");
    let debug = &verified.executable().functions[0].debug;
    assert_eq!(debug.sequence_points.len(), 2);
    assert!(debug.bindings.is_empty());
    assert!(debug.capture_sources.is_empty());
    assert_eq!(debug.lexical_owner, None);
    assert_eq!(debug.result_type, None);
}

#[test]
fn opcode_validation_does_not_record_register_types_or_init_masks() {
    let verified = all_opcodes_executable().verify().expect("opcode fixture");
    for function in &verified.executable().functions {
        assert!(
            function.debug.sequence_points.is_empty(),
            "opcode validation succeeds without sequence-point dataflow facts"
        );
        assert!(function.debug.bindings.is_empty());
    }
}
