use super::*;

#[test]
fn positional_record_clones_share_layout_and_detach_values() {
    let mut image = unverified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeRecord, 1, 0, 0),
            abc(Opcode::Move, 2, 1, 0),
            abx(Opcode::LoadConstant, 3, 1),
            abc(Opcode::StoreField, 2, 0, 3),
            return_unit(),
        ],
        vec![Constant::Integer(1), Constant::Integer(2)],
        vec!["root", "test.fpas", "Point", "x"],
        4,
    );
    image.records = vec![RecordLayout {
        construction: None,
        name: StringId::new(2),
        fields: vec![RecordField {
            name: StringId::new(3),
            ty: crate::DebugTypeId::new(0),
            type_name: None,
        }],
        methods: Vec::new(),
    }];
    let (_, registers, _) = execute(image.verify().expect("record image must verify"))
        .expect("record program must run");
    let (Value::Record(original), Value::Record(updated)) = (&registers[1], &registers[2]) else {
        panic!("expected positional records");
    };
    assert!(std::sync::Arc::ptr_eq(
        &original.body().layout,
        &updated.body().layout
    ));
    assert_eq!(original.body().values[0], Value::Integer(1));
    assert_eq!(updated.body().values[0], Value::Integer(2));
}

#[test]
fn store_field_reuses_unique_storage() {
    let mut image = unverified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeRecord, 1, 0, 0),
            abc(Opcode::StoreField, 1, 0, 0),
            return_unit(),
        ],
        vec![Constant::Integer(7)],
        vec!["root", "test.fpas", "Point", "x"],
        2,
    );
    image.records = vec![RecordLayout {
        construction: None,
        name: StringId::new(2),
        fields: vec![RecordField {
            name: StringId::new(3),
            ty: crate::DebugTypeId::new(0),
            type_name: None,
        }],
        methods: Vec::new(),
    }];
    let mut worker =
        crate::execution::vm::worker::Worker::new(Arc::new(image.verify().expect("record image")))
            .expect("worker");
    worker.dispatch_one().expect("constant");
    worker.dispatch_one().expect("record");
    let Value::Record(record) = &worker.registers[1] else {
        panic!("record");
    };
    let storage = record.body().values.as_ptr();
    worker.dispatch_one().expect("field store");
    let Value::Record(record) = &worker.registers[1] else {
        panic!("record");
    };
    assert_eq!(record.body().values.as_ptr(), storage);
    assert_eq!(record.body().values[0], Value::Integer(7));
    assert!(worker.register_is_initialized(1));
}

#[test]
fn store_field_from_its_own_record_preserves_the_previous_value() {
    let image = point_image(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeRecord, 1, 0, 0),
            abc(Opcode::StoreField, 1, 0, 1),
            abc(Opcode::LoadField, 2, 1, 0),
            abc(Opcode::LoadField, 2, 2, 0),
            return_unit(),
        ],
        vec![Constant::Integer(7)],
        3,
    );
    let (_, registers, _) = execute(image).expect("self-store runs");
    let Value::Record(outer) = &registers[1] else {
        panic!("outer record");
    };
    let Value::Record(inner) = &outer.body().values[0] else {
        panic!("stored record snapshot");
    };
    assert_eq!(inner.body().values[0], Value::Integer(7));
    assert_eq!(registers[2], Value::Integer(7));
}

#[test]
fn load_field_over_its_source_releases_the_record() {
    for (constant, expected) in [
        (Constant::Integer(7), Value::Integer(7)),
        (Constant::Real((-0.0_f64).to_bits()), Value::Real(-0.0)),
        (Constant::Boolean(true), Value::Boolean(true)),
        (Constant::String(StringId::new(3)), Value::Str("x".into())),
    ] {
        let image = point_image(
            vec![
                abx(Opcode::LoadConstant, 0, 0),
                abc(Opcode::MakeRecord, 1, 0, 0),
                abc(Opcode::LoadField, 1, 1, 0),
                return_unit(),
            ],
            vec![constant],
            2,
        );
        let mut worker =
            crate::execution::vm::worker::Worker::new(Arc::new(image)).expect("worker");
        let layout = worker.layouts.records[0].clone();
        let references = Arc::strong_count(&layout);
        worker.dispatch_one().expect("constant");
        worker.dispatch_one().expect("record");
        assert_eq!(Arc::strong_count(&layout), references + 1);
        worker.dispatch_one().expect("field load over source");
        assert_eq!(Arc::strong_count(&layout), references);
        assert_eq!(worker.registers[1], expected);
        if let Value::Real(value) = worker.registers[1] {
            assert_eq!(value.to_bits(), (-0.0_f64).to_bits());
        }
        assert!(worker.register_is_initialized(1));
    }
}

#[test]
fn invalid_field_store_preserves_the_destination() {
    let image = point_image(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeRecord, 1, 0, 0),
            return_unit(),
        ],
        vec![Constant::Integer(7)],
        2,
    );
    let mut worker = crate::execution::vm::worker::Worker::new(Arc::new(image)).expect("worker");
    worker.dispatch_one().expect("constant");
    worker.dispatch_one().expect("record");
    let before = worker.registers[1].clone();
    let error = worker
        .store_field(crate::AbcOperands {
            a: 1,
            b: 1,
            c: 0,
            auxiliary: 0,
        })
        .expect_err("invalid field");
    assert_eq!(error.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
    assert_eq!(error.message, "Verified record field slot 1 is unavailable");
    assert_eq!(worker.registers[1], before);
    assert!(worker.register_is_initialized(1));
}

/// Verified single-field `Point` record image for update tests.
fn point_image(
    code: Vec<crate::Instruction>,
    constants: Vec<Constant>,
    register_count: u16,
) -> crate::VerifiedExecutable {
    let mut image = unverified(
        code,
        constants,
        vec!["root", "test.fpas", "Point", "x"],
        register_count,
    );
    image.records = vec![RecordLayout {
        construction: None,
        name: StringId::new(2),
        fields: vec![RecordField {
            name: StringId::new(3),
            ty: crate::DebugTypeId::new(0),
            type_name: None,
        }],
        methods: Vec::new(),
    }];
    image.verify().expect("record image must verify")
}

#[test]
fn update_record_preserves_aliases() {
    let image = point_image(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeRecord, 1, 0, 0),
            abc(Opcode::Move, 2, 1, 0),
            abx(Opcode::LoadConstant, 3, 1),
            abx(Opcode::LoadConstant, 4, 2),
            abc(Opcode::UpdateRecord, 1, 3, 1),
            return_unit(),
        ],
        vec![
            Constant::Integer(7),
            Constant::Integer(0),
            Constant::Integer(9),
        ],
        5,
    );
    let (_, registers, _) = execute(image).expect("record update runs");
    let (Value::Record(updated), Value::Record(alias)) = (&registers[1], &registers[2]) else {
        panic!("expected records");
    };
    assert_eq!(updated.body().values[0], Value::Integer(9));
    assert_eq!(alias.body().values[0], Value::Integer(7));
}

#[test]
fn update_record_reuses_unique_storage() {
    let image = point_image(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeRecord, 1, 0, 0),
            abx(Opcode::LoadConstant, 2, 1),
            abx(Opcode::LoadConstant, 3, 2),
            abc(Opcode::UpdateRecord, 1, 2, 1),
            return_unit(),
        ],
        vec![
            Constant::Integer(7),
            Constant::Integer(0),
            Constant::Integer(9),
        ],
        4,
    );
    let mut worker = crate::execution::vm::worker::Worker::new(Arc::new(image)).expect("worker");
    for _ in 0..4 {
        worker.dispatch_one().expect("setup");
    }
    let Value::Record(record) = &worker.registers[1] else {
        panic!("record");
    };
    let storage = record.body().values.as_ptr();
    worker.dispatch_one().expect("record update");
    let Value::Record(record) = &worker.registers[1] else {
        panic!("record");
    };
    assert_eq!(record.body().values.as_ptr(), storage);
    assert_eq!(record.body().values[0], Value::Integer(9));
    assert!(worker.register_is_initialized(1));
}
