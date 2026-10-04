//! Canonical reference words and positional var modes in executable metadata.

use fpas_bytecode::{Opcode, ValidationErrorKind};

use super::support::*;

fn parameter_image(arity: u8, positions: Vec<u8>) -> fpas_bytecode::Executable {
    let mut image = minimal_executable();
    image.code.push(return_unit());
    let mut callee = image.functions[0].clone();
    callee.code = fpas_bytecode::CodeRange::new(
        fpas_bytecode::InstructionAddress::new(1),
        fpas_bytecode::InstructionAddress::new(2),
    );
    callee.arity = arity;
    callee.register_count = u16::from(arity);
    callee.var_parameters = positions;
    image.functions.push(callee);
    let mut source = image.source_map.runs[0];
    source.instruction_start = fpas_bytecode::InstructionAddress::new(1);
    image.source_map.runs.push(source);
    image
}

#[test]
fn var_positions_are_strictly_ordered_distinct_and_in_range() {
    for positions in [vec![0, 0], vec![1, 0], vec![2], vec![u8::MAX]] {
        let image = parameter_image(2, positions);
        assert!(
            matches!(image.verify(), Err(error) if matches!(error.kind, ValidationErrorKind::ParameterModes { .. }))
        );
    }
    let image = parameter_image(u8::MAX, vec![0, u8::MAX - 1]);
    assert!(image.verify().is_ok());
}

#[test]
fn reference_words_reject_reserved_fields_invalid_registers_and_layout_slots() {
    for word in [
        abc(Opcode::ReserveReference, 0, 1, 1, 0),
        abc(Opcode::ReadReference, 0, 1, 0, 1),
        abc(Opcode::WriteReference, 0, 1, 2, 0),
        abc(Opcode::ReleaseReference, 0, 1, 0, 0),
        abc(Opcode::SelectReferenceIndex, 0, 1, 16, 0),
        abc(Opcode::SelectReferenceField, 0, 1, 0, 0),
        abc(Opcode::SelectReferenceField, 0, 0, 1, 0),
    ] {
        let mut image = all_opcodes_executable();
        image.code[0] = word;
        assert!(image.verify().is_err(), "must reject {word:?}");
    }
}
