use fpas_bytecode::{
    CodeRange, Constant, EnumTypeId, FunctionId, Instruction, InstructionAddress, NO_REGISTER,
    Opcode, ReturnConvention, SourceId, StringId, ValidationErrorKind,
};

use super::support::{
    abc, abx, all_opcodes_executable, minimal_executable, replace_root_code, return_unit,
};

fn error_kind(executable: fpas_bytecode::Executable) -> ValidationErrorKind {
    executable
        .verify()
        .expect_err("fixture must be rejected")
        .kind
}

fn opcode_index(executable: &fpas_bytecode::Executable, opcode: Opcode) -> usize {
    executable
        .code
        .iter()
        .position(|instruction| instruction.opcode() == Ok(opcode))
        .expect("fixture must contain opcode")
}

#[test]
fn unknown_and_reserved_opcodes_are_rejected() {
    let mut unknown = minimal_executable();
    replace_root_code(
        &mut unknown,
        vec![Instruction::from_word(u64::from(u8::MAX))],
    );
    assert!(matches!(
        error_kind(unknown),
        ValidationErrorKind::Instruction(fpas_bytecode::InstructionError::UnknownOpcode(_))
    ));

    let mut reserved = minimal_executable();
    replace_root_code(
        &mut reserved,
        vec![Instruction::ax(Opcode::ReservedMetadata, 0).expect("Ax fixture")],
    );
    assert_eq!(error_kind(reserved), ValidationErrorKind::ReservedOpcode);
}

#[test]
fn register_sentinel_and_frame_bound_are_rejected() {
    for invalid in [NO_REGISTER, 1] {
        let mut executable = minimal_executable();
        executable.functions[0].register_count = 1;
        replace_root_code(
            &mut executable,
            vec![abc(Opcode::LoadUnit, invalid, 0, 0, 0), return_unit()],
        );
        assert!(matches!(
            error_kind(executable),
            ValidationErrorKind::Register {
                operand: "destination",
                actual,
                register_count: 1
            } if actual == invalid
        ));
    }
}

#[test]
fn constant_global_function_and_intrinsic_ids_are_rejected() {
    let mut constant = minimal_executable();
    constant.functions[0].register_count = 1;
    replace_root_code(
        &mut constant,
        vec![abx(Opcode::LoadConstant, 0, 0), return_unit()],
    );
    assert!(matches!(
        error_kind(constant),
        ValidationErrorKind::TableReference {
            table: "constants",
            ..
        }
    ));

    let mut global = minimal_executable();
    global.functions[0].register_count = 1;
    replace_root_code(
        &mut global,
        vec![abx(Opcode::LoadGlobal, 0, 0), return_unit()],
    );
    assert!(matches!(
        error_kind(global),
        ValidationErrorKind::TableReference {
            table: "globals",
            ..
        }
    ));

    let mut function = minimal_executable();
    replace_root_code(
        &mut function,
        vec![abc(Opcode::CallDirect, NO_REGISTER, 1, 0, 0), return_unit()],
    );
    assert!(matches!(
        error_kind(function),
        ValidationErrorKind::TableReference {
            table: "functions",
            ..
        }
    ));

    let mut intrinsic = minimal_executable();
    replace_root_code(
        &mut intrinsic,
        vec![
            abc(Opcode::Intrinsic, NO_REGISTER, u16::MAX, 0, 0),
            return_unit(),
        ],
    );
    assert_eq!(
        error_kind(intrinsic),
        ValidationErrorKind::UnknownIntrinsic { actual: u16::MAX }
    );
}

#[test]
fn call_arity_destination_and_argument_windows_are_rejected() {
    let mut arity = all_opcodes_executable();
    let call = opcode_index(&arity, Opcode::CallDirect);
    arity.code[call] = abc(Opcode::CallDirect, NO_REGISTER, 1, 0, 1);
    assert!(matches!(
        error_kind(arity),
        ValidationErrorKind::CallArity {
            expected: 0,
            actual: 1,
            ..
        }
    ));

    let mut destination = all_opcodes_executable();
    let call = opcode_index(&destination, Opcode::CallDirect);
    destination.code[call] = abc(Opcode::CallDirect, 0, 1, 0, 0);
    assert!(matches!(
        error_kind(destination),
        ValidationErrorKind::ReturnConvention {
            expected: ReturnConvention::Unit,
            actual: 0
        }
    ));

    let mut window = minimal_executable();
    window.functions[0].register_count = 1;
    replace_root_code(
        &mut window,
        vec![abc(Opcode::CallValue, NO_REGISTER, 0, 1, 1), return_unit()],
    );
    assert!(matches!(
        error_kind(window),
        ValidationErrorKind::RegisterWindow {
            base: 1,
            count: 1,
            ..
        }
    ));
}

#[test]
fn function_ranges_partitions_and_frames_are_rejected() {
    let mut entry = minimal_executable();
    entry.entry = FunctionId::new(1);
    assert!(matches!(
        error_kind(entry),
        ValidationErrorKind::EntryFunction { .. }
    ));

    let mut empty = minimal_executable();
    empty.functions[0].code =
        CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(0));
    assert!(matches!(
        error_kind(empty),
        ValidationErrorKind::EmptyCodeRange { .. }
    ));

    let mut outside = minimal_executable();
    outside.functions[0].code =
        CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(2));
    assert!(matches!(
        error_kind(outside),
        ValidationErrorKind::CodeRange { .. }
    ));

    let mut root_signature = minimal_executable();
    root_signature.functions[0].arity = 1;
    assert!(matches!(
        error_kind(root_signature),
        ValidationErrorKind::EntrySignature { .. }
    ));

    let mut frame = all_opcodes_executable();
    frame.functions[1].arity = 1;
    let direct_call = opcode_index(&frame, Opcode::CallDirect);
    frame.code[direct_call] = abc(Opcode::CallDirect, NO_REGISTER, 1, 0, 1);
    assert!(matches!(
        error_kind(frame),
        ValidationErrorKind::FrameWindow { .. }
    ));

    let mut captures = all_opcodes_executable();
    captures.constants.truncate(2);
    captures.functions[1].capture_count = 256;
    captures.functions[1].register_count = 256;
    let closure = opcode_index(&captures, Opcode::MakeClosure);
    captures.code[closure] = abc(Opcode::LoadUnit, 0, 0, 0, 0);
    assert!(matches!(
        error_kind(captures),
        ValidationErrorKind::FrameWindow { captures: 256, .. }
    ));

    let mut overlap = all_opcodes_executable();
    let second_start = overlap.functions[1].code.start.get();
    overlap.functions[1].code.start = InstructionAddress::new(second_start - 1);
    assert!(matches!(
        error_kind(overlap),
        ValidationErrorKind::FunctionPartition { .. }
    ));
}

#[test]
fn closure_capture_count_must_match_target_metadata() {
    let mut executable = all_opcodes_executable();
    let closure = opcode_index(&executable, Opcode::MakeClosure);
    executable.code[closure] = abc(Opcode::MakeClosure, 0, 1, 0, 1);
    assert!(matches!(
        error_kind(executable),
        ValidationErrorKind::RegisterWindow { .. }
    ));
}

#[test]
fn branches_cannot_leave_a_function_and_reachable_code_cannot_fall_off() {
    let mut branch = minimal_executable();
    replace_root_code(&mut branch, vec![abx(Opcode::Jump, 0, 1)]);
    assert!(matches!(
        error_kind(branch),
        ValidationErrorKind::BranchTarget { .. }
    ));

    let mut fallthrough = minimal_executable();
    fallthrough.functions[0].register_count = 1;
    replace_root_code(&mut fallthrough, vec![abc(Opcode::LoadUnit, 0, 0, 0, 0)]);
    assert_eq!(error_kind(fallthrough), ValidationErrorKind::Fallthrough);
}

#[test]
fn returns_must_follow_function_metadata() {
    let mut value = all_opcodes_executable();
    value.functions[1].return_convention = ReturnConvention::Value;
    value.functions[1].register_count = 1;
    let direct_call = opcode_index(&value, Opcode::CallDirect);
    value.code[direct_call] = abc(Opcode::CallDirect, 0, 1, 0, 0);
    assert!(matches!(
        error_kind(value),
        ValidationErrorKind::ReturnConvention {
            expected: ReturnConvention::Value,
            actual: NO_REGISTER
        }
    ));
}

#[test]
fn string_constant_and_layout_metadata_references_are_rejected() {
    let mut duplicate = minimal_executable();
    duplicate.strings = fpas_bytecode::StringTable::new(vec!["root".into(), "root".into()]);
    assert!(matches!(
        error_kind(duplicate),
        ValidationErrorKind::DuplicateString { .. }
    ));

    let mut string = minimal_executable();
    string.functions[0].name = StringId::new(99);
    assert!(matches!(
        error_kind(string),
        ValidationErrorKind::StringReference {
            owner: "function name",
            ..
        }
    ));

    let mut constant = minimal_executable();
    constant.constants.push(Constant::Function {
        function: FunctionId::new(9),
        task_bound: false,
    });
    assert!(matches!(
        error_kind(constant),
        ValidationErrorKind::TableReference {
            operand: "constant function",
            ..
        }
    ));

    let mut task_bound_constant = all_opcodes_executable();
    task_bound_constant.constants[2] = Constant::Function {
        function: FunctionId::new(1),
        task_bound: true,
    };
    assert!(matches!(
        error_kind(task_bound_constant),
        ValidationErrorKind::ConstantFunction {
            function: 1,
            captures: 0,
            task_bound: true
        }
    ));

    let mut variant = all_opcodes_executable();
    variant.enum_variants[0].owner = EnumTypeId::new(1);
    assert!(matches!(
        error_kind(variant),
        ValidationErrorKind::TableReference {
            operand: "variant owner",
            ..
        }
    ));
}

#[test]
fn aggregate_type_field_and_variant_operands_are_rejected() {
    let mut record = all_opcodes_executable();
    let make = opcode_index(&record, Opcode::MakeRecord);
    record.code[make] = abc(Opcode::MakeRecord, 0, 1, 0, 0);
    assert!(matches!(
        error_kind(record),
        ValidationErrorKind::TableReference {
            table: "record layouts",
            ..
        }
    ));

    let mut field = all_opcodes_executable();
    let load = opcode_index(&field, Opcode::LoadField);
    field.code[load] = abc(Opcode::LoadField, 0, 1, 1, 0);
    assert!(matches!(
        error_kind(field),
        ValidationErrorKind::LayoutReference {
            operand: "record field",
            ..
        }
    ));

    let mut variant = all_opcodes_executable();
    let test = opcode_index(&variant, Opcode::TestVariant);
    variant.code[test] = abc(Opcode::TestVariant, 0, 1, 1, 0);
    assert!(matches!(
        error_kind(variant),
        ValidationErrorKind::TableReference {
            table: "enum variants",
            ..
        }
    ));
}

#[test]
fn source_runs_must_be_ordered_bounded_and_present_at_function_boundaries() {
    let mut order = all_opcodes_executable();
    order.source_map.runs[1].instruction_start = InstructionAddress::new(0);
    assert!(matches!(
        error_kind(order),
        ValidationErrorKind::SourceRunOrder { .. }
    ));

    let mut address = minimal_executable();
    address.source_map.runs[0].instruction_start = InstructionAddress::new(1);
    assert!(matches!(
        error_kind(address),
        ValidationErrorKind::SourceRunAddress { .. }
    ));

    let mut source = minimal_executable();
    source.source_map.runs[0].source = SourceId::new(1);
    assert!(matches!(
        error_kind(source),
        ValidationErrorKind::SourceReference { .. }
    ));

    let mut position = minimal_executable();
    position.source_map.runs[0].line = 0;
    assert!(matches!(
        error_kind(position),
        ValidationErrorKind::SourcePosition { .. }
    ));

    let mut boundary = minimal_executable();
    boundary.source_map.runs.clear();
    assert!(matches!(
        error_kind(boundary),
        ValidationErrorKind::MissingFunctionSource { .. }
    ));
}

#[test]
fn spawn_flags_and_canonical_unused_operands_must_match_code() {
    let mut spawn = all_opcodes_executable();
    spawn.functions[0].flags.uses_spawn_tasks = false;
    assert!(matches!(
        error_kind(spawn),
        ValidationErrorKind::SpawnFlag {
            declared: false,
            emitted: true
        }
    ));

    let mut canonical = minimal_executable();
    replace_root_code(
        &mut canonical,
        vec![abc(Opcode::Return, NO_REGISTER, 1, 0, 0)],
    );
    assert!(matches!(
        error_kind(canonical),
        ValidationErrorKind::NonCanonicalOperand {
            operand: "B",
            actual: 1,
            expected: 0
        }
    ));
}
