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
        name: StringId::new(2),
        fields: vec![RecordField {
            name: StringId::new(3),
            ty: fpas_bytecode::DebugTypeId::new(0),
        }],
        properties: Vec::new(),
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
        name: StringId::new(2),
        fields: vec![RecordField {
            name: StringId::new(3),
            ty: fpas_bytecode::DebugTypeId::new(0),
        }],
        properties: Vec::new(),
        methods: Vec::new(),
    }];
    let mut worker =
        crate::vm::worker::Worker::new(Arc::new(image.verify().expect("record image")))
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
}
