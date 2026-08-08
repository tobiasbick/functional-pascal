#[test]
#[expect(clippy::expect_used, reason = "fixture shape failures should stop the test")]
fn p5_collection_types_reject_wrong_values_and_string_updates() {
    let mut wrong_array_value = all_operations_program();
    let instruction = wrong_array_value.functions[0].blocks[0]
        .instructions
        .iter_mut()
        .find(|instruction| matches!(instruction.operation, Operation::IndexSet { .. }))
        .expect("fixture contains IndexSet");
    instruction.operation = Operation::IndexSet {
        collection: ValueId::new(18),
        index: ValueId::new(1),
        value: ValueId::new(2),
    };
    assert!(wrong_array_value.validate().is_err());

    let mut string_update = all_operations_program();
    let instruction = string_update.functions[0].blocks[0]
        .instructions
        .iter_mut()
        .find(|instruction| matches!(instruction.operation, Operation::IndexSet { .. }))
        .expect("fixture contains IndexSet");
    instruction.result = Some(value(20, STRING));
    instruction.operation = Operation::IndexSet {
        collection: ValueId::new(3),
        index: ValueId::new(1),
        value: ValueId::new(3),
    };
    assert!(string_update.validate().is_err());
}

#[test]
#[expect(clippy::expect_used, reason = "fixture shape failures should stop the test")]
fn p5_wrapper_tests_and_payloads_require_matching_types() {
    let mut wrong_test = all_operations_program();
    let instruction = wrong_test.functions[0].blocks[0]
        .instructions
        .iter_mut()
        .find(|instruction| matches!(instruction.operation, Operation::IsResultOk(_)))
        .expect("fixture contains IsResultOk");
    instruction.operation = Operation::IsResultOk(ValueId::new(29));
    assert!(wrong_test.validate().is_err());

    let mut wrong_payload = all_operations_program();
    let instruction = wrong_payload.functions[0].blocks[0]
        .instructions
        .iter_mut()
        .find(|instruction| matches!(instruction.operation, Operation::MakeSome(_)))
        .expect("fixture contains MakeSome");
    instruction.operation = Operation::MakeSome(ValueId::new(3));
    assert!(wrong_payload.validate().is_err());

    let mut wrong_unwrap = all_operations_program();
    let instruction = wrong_unwrap.functions[0].blocks[0]
        .instructions
        .iter_mut()
        .find(|instruction| matches!(instruction.operation, Operation::UnwrapSome(_)))
        .expect("fixture contains UnwrapSome");
    instruction.result = Some(value(32, STRING));
    assert!(wrong_unwrap.validate().is_err());
}

#[test]
#[expect(clippy::expect_used, reason = "fixture shape failures should stop the test")]
fn p5_record_and_enum_slots_reject_unknown_fields() {
    let mut record = all_operations_program();
    let instruction = record.functions[0].blocks[0]
        .instructions
        .iter_mut()
        .find(|instruction| matches!(instruction.operation, Operation::UpdateRecord { .. }))
        .expect("fixture contains UpdateRecord");
    instruction.operation = Operation::UpdateRecord {
        record: ValueId::new(10),
        layout: RecordLayoutId::new(0),
        fields: vec![(FieldId::new(99), ValueId::new(1))],
    };
    assert!(record.validate().is_err());

    let mut enumeration = all_operations_program();
    let instruction = enumeration.functions[0].blocks[0]
        .instructions
        .iter_mut()
        .find(|instruction| matches!(instruction.operation, Operation::LoadEnumField { .. }))
        .expect("fixture contains LoadEnumField");
    instruction.operation = Operation::LoadEnumField {
        value: ValueId::new(12),
        layout: EnumLayoutId::new(0),
        variant: VariantId::new(0),
        field: FieldId::new(99),
    };
    assert!(enumeration.validate().is_err());
}
