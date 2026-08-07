use std::mem::size_of;

use fpas_bytecode::{
    Instruction, InstructionError, InstructionForm, Opcode, OperandError, Register, Value,
};

#[test]
fn every_opcode_round_trips_through_its_declared_form() {
    for opcode in Opcode::ALL {
        let instruction = match opcode.form() {
            InstructionForm::Abc => Instruction::abc(opcode, 0x1234, 0x5678, 0x9abc, 0xde),
            InstructionForm::Abx => Instruction::abx(opcode, 0x1234, 0x9abcdef0),
            InstructionForm::Ax => Instruction::ax(opcode, 0x1234_5678_9abc),
        }
        .expect("declared form must construct");
        assert_eq!(instruction.opcode(), Ok(opcode));
        match opcode.form() {
            InstructionForm::Abc => assert_eq!(
                instruction.abc_operands().expect("ABC must decode"),
                fpas_bytecode::AbcOperands {
                    a: 0x1234,
                    b: 0x5678,
                    c: 0x9abc,
                    auxiliary: 0xde,
                }
            ),
            InstructionForm::Abx => assert_eq!(
                instruction.abx_operands().expect("ABx must decode"),
                fpas_bytecode::AbxOperands {
                    a: 0x1234,
                    bx: 0x9abcdef0,
                }
            ),
            InstructionForm::Ax => assert_eq!(
                instruction.ax_operand().expect("Ax must decode"),
                0x1234_5678_9abc
            ),
        }
    }
}

#[test]
fn opcode_inventory_is_exhaustive_and_contiguous() {
    let decoded: Vec<_> = (u8::MIN..=u8::MAX)
        .filter_map(|raw| Opcode::try_from(raw).ok())
        .collect();
    assert_eq!(decoded, Opcode::ALL);
    assert_eq!(Opcode::ALL.len(), 94);
}

#[test]
fn malformed_instruction_forms_and_unknown_opcodes_are_rejected() {
    assert!(matches!(
        Instruction::abc(Opcode::Jump, 0, 0, 0, 0),
        Err(InstructionError::FormMismatch { .. })
    ));
    assert!(matches!(
        Instruction::ax(Opcode::ReservedMetadata, 1_u64 << 48),
        Err(InstructionError::PayloadOutOfRange { .. })
    ));
    assert_eq!(
        Instruction::from_word(u64::from(u8::MAX)).opcode(),
        Err(InstructionError::UnknownOpcode(u8::MAX))
    );
}

#[test]
fn packed_instruction_and_runtime_value_sizes_meet_the_contract() {
    assert_eq!(size_of::<Instruction>(), 8);
    assert!(
        size_of::<Value>() <= 16,
        "Value grew to {} bytes",
        size_of::<Value>()
    );
}

#[test]
fn register_sentinel_and_index_conversion_are_checked() {
    assert_eq!(Register::MAX.get(), u16::MAX - 1);
    assert_eq!(Register::new(u16::MAX), Err(OperandError::ReservedRegister));
    assert!(matches!(
        Register::try_from_index(usize::from(u16::MAX)),
        Err(OperandError::ReservedRegister)
    ));
    assert_eq!(
        Register::try_from_index(usize::from(u16::MAX) - 1).expect("largest register must fit"),
        Register::MAX
    );
}
