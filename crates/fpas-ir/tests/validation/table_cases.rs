fn assert_positional_error(
    program: &Program,
    entity: fpas_ir::validate::EntityKind,
    expected: u32,
    actual: u32,
) {
    assert!(matches!(
        program.validate(),
        Err(error)
            if error.kind
                == fpas_ir::validate::ValidationErrorKind::PositionalId {
                    entity,
                    expected,
                    actual,
                }
    ));
}

#[test]
fn positional_global_ids_must_match_their_table_index() {
    let mut program = all_operations_program();
    program.globals.swap(0, 1);

    assert_positional_error(
        &program,
        fpas_ir::validate::EntityKind::Global,
        0,
        1,
    );
}

#[test]
fn positional_program_table_ids_must_match_their_indexes() {
    let mut types = scalar_program();
    types.types.swap(0, 1);
    assert_positional_error(&types, fpas_ir::validate::EntityKind::Type, 0, 1);

    let mut records = scalar_program();
    let mut second_record = records.record_layouts[0].clone();
    second_record.id = RecordLayoutId::new(1);
    records.record_layouts.push(second_record);
    records.record_layouts.swap(0, 1);
    assert_positional_error(
        &records,
        fpas_ir::validate::EntityKind::RecordLayout,
        0,
        1,
    );

    let mut enums = scalar_program();
    let mut second_enum = enums.enum_layouts[0].clone();
    second_enum.id = EnumLayoutId::new(1);
    enums.enum_layouts.push(second_enum);
    enums.enum_layouts.swap(0, 1);
    assert_positional_error(
        &enums,
        fpas_ir::validate::EntityKind::EnumLayout,
        0,
        1,
    );

    let mut functions = all_operations_program();
    functions.functions.swap(0, 1);
    assert_positional_error(
        &functions,
        fpas_ir::validate::EntityKind::Function,
        0,
        1,
    );
}

#[test]
fn positional_member_ids_must_match_their_indexes() {
    let mut records = scalar_program();
    let mut second_field = records.record_layouts[0].fields[0].clone();
    second_field.id = FieldId::new(1);
    records.record_layouts[0].fields.push(second_field);
    records.record_layouts[0].fields.swap(0, 1);
    assert_positional_error(&records, fpas_ir::validate::EntityKind::Field, 0, 1);

    let mut enums = scalar_program();
    let mut second_variant = enums.enum_layouts[0].variants[0].clone();
    second_variant.id = VariantId::new(1);
    enums.enum_layouts[0].variants.push(second_variant);
    enums.enum_layouts[0].variants.swap(0, 1);
    assert_positional_error(&enums, fpas_ir::validate::EntityKind::Variant, 0, 1);
}

#[test]
fn positional_ids_must_not_contain_gaps() {
    let mut program = all_operations_program();
    program.globals[1].id = GlobalId::new(2);

    assert_positional_error(
        &program,
        fpas_ir::validate::EntityKind::Global,
        1,
        2,
    );
}

#[test]
fn basic_block_ids_remain_identity_based_and_keep_declared_order() {
    let mut program = scalar_program();
    program.functions[0].blocks = vec![
        BasicBlock {
            id: BlockId::new(10),
            parameters: Vec::new(),
            instructions: Vec::new(),
            terminators: vec![Terminator::Return(None)],
        },
        BasicBlock {
            id: BlockId::new(0),
            parameters: Vec::new(),
            instructions: Vec::new(),
            terminators: vec![Terminator::Jump(BlockTarget {
                block: BlockId::new(10),
                arguments: Vec::new(),
            })],
        },
    ];

    assert!(program.validate().is_ok());
    assert_eq!(
        program.functions[0]
            .blocks
            .iter()
            .map(|block| block.id.get())
            .collect::<Vec<_>>(),
        vec![10, 0]
    );
}
