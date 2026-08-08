use fpas_bytecode::{
    Constant, EnumLayout, EnumTypeId, EnumVariant, GlobalInfo, Opcode, RecordField, RecordLayout,
    StringId, Value,
};
use fpas_diagnostics::codes::{
    RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS, RUNTIME_DICT_KEY_NOT_FOUND, RUNTIME_VM_OPERAND_TYPE_MISMATCH,
};

use super::*;

#[test]
fn arrays_use_copy_on_write_for_index_updates() {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::MakeArray, 2, 0, 2),
            abc(Opcode::Move, 3, 2, 0),
            abx(Opcode::LoadConstant, 4, 2),
            abc(Opcode::IndexSet, 3, 0, 4),
            abc(Opcode::IndexGet, 5, 2, 0),
            abc(Opcode::IndexGet, 6, 3, 0),
            return_unit(),
        ],
        vec![
            Constant::Integer(0),
            Constant::Integer(2),
            Constant::Integer(9),
        ],
        vec!["root", "test.fpas"],
        7,
    );
    let (_, registers, _) = execute(executable).expect("aggregate program must run");
    assert_eq!(registers[5], Value::Integer(0));
}

#[test]
fn array_push_consumes_unique_storage_and_preserves_aliases() {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abx(Opcode::LoadConstant, 2, 2),
            abc(Opcode::MakeArray, 3, 1, 1),
            abc(Opcode::Move, 4, 3, 0),
            abc(Opcode::ArrayPush, 3, 3, 2),
            abc(Opcode::IndexGet, 5, 4, 0),
            abc(Opcode::IndexGet, 6, 3, 1),
            return_unit(),
        ],
        vec![
            Constant::Integer(0),
            Constant::Integer(1),
            Constant::Integer(2),
        ],
        vec!["root", "test.fpas"],
        7,
    );
    let (_, registers, _) = execute(executable).expect("array push must run");
    assert_eq!(registers[5], Value::Integer(1));
    assert_eq!(registers[6], Value::Integer(2));
    let (Value::Array(original), Value::Array(updated)) = (&registers[4], &registers[3]) else {
        panic!("expected array values");
    };
    assert_eq!(original.len(), 1);
    assert_eq!(updated.len(), 2);
}

#[test]
fn array_push_rejects_non_array_operands() {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::ArrayPush, 2, 0, 1),
            return_unit(),
        ],
        vec![Constant::Integer(1), Constant::Integer(2)],
        vec!["root", "test.fpas"],
        3,
    );
    let error = execute(executable).expect_err("non-array push must fail");
    assert_eq!(error.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
    assert_eq!(error.message, "Expected array, got integer");
}

#[test]
fn globals_are_dense_and_immutable_slots_initialize_once() {
    let mut image = unverified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::StoreGlobal, 0, 0),
            abx(Opcode::LoadGlobal, 1, 0),
            return_unit(),
        ],
        vec![Constant::Integer(42)],
        vec!["root", "test.fpas", "answer"],
        2,
    );
    image.globals = vec![GlobalInfo {
        name: StringId::new(2),
        mutable: false,
    }];
    let (_, registers, _) = execute(image.verify().expect("global image must verify"))
        .expect("global program must run");
    assert_eq!(registers[1], Value::Integer(42));
}

#[test]
fn record_and_enum_slots_are_positional() {
    let mut image = unverified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeRecord, 1, 0, 0),
            abc(Opcode::LoadField, 2, 1, 0),
            abc(Opcode::MakeEnum, 3, 0, 2),
            abc(Opcode::TestVariant, 4, 3, 0),
            abc(Opcode::LoadEnumField, 5, 3, 0),
            return_unit(),
        ],
        vec![Constant::Integer(7)],
        vec!["root", "test.fpas", "Point", "x", "Choice", "Some", "value"],
        6,
    );
    image.records = vec![RecordLayout {
        name: StringId::new(2),
        fields: vec![RecordField {
            name: StringId::new(3),
        }],
    }];
    image.enums = vec![EnumLayout {
        name: StringId::new(4),
    }];
    image.enum_variants = vec![EnumVariant {
        owner: EnumTypeId::new(0),
        name: StringId::new(5),
        fields: vec![StringId::new(6)],
    }];
    let (_, registers, _) = execute(image.verify().expect("layout image must verify"))
        .expect("layout program must run");
    assert_eq!(registers[5], Value::Integer(7));
    assert_eq!(registers[2], Value::Integer(7));
    assert_eq!(registers[4], Value::Boolean(true));
}

#[test]
fn result_and_option_operations_preserve_payloads() {
    let executable = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abc(Opcode::MakeSome, 1, 0, 0),
            abc(Opcode::IsOptionSome, 2, 1, 0),
            abc(Opcode::UnwrapSome, 3, 1, 0),
            return_unit(),
        ],
        vec![Constant::Integer(5)],
        vec!["root", "test.fpas"],
        4,
    );
    let (_, registers, _) = execute(executable).expect("option program must run");
    assert_eq!(registers[3], Value::Integer(5));
    assert_eq!(registers[2], Value::Boolean(true));
}

#[test]
fn unwrap_rejects_the_wrong_result_or_option_variant() {
    let executable = verified(
        vec![
            abc(Opcode::MakeNone, 0, 0, 0),
            abc(Opcode::UnwrapSome, 1, 0, 0),
            return_unit(),
        ],
        Vec::new(),
        vec!["root", "test.fpas"],
        2,
    );
    let error = execute(executable).expect_err("None must not unwrap as Some");
    assert_eq!(error.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
    assert_eq!(error.message, "Cannot unwrap Some from this value");
}

#[test]
fn immutable_global_rejects_a_second_store() {
    let mut image = unverified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::StoreGlobal, 0, 0),
            abx(Opcode::StoreGlobal, 0, 0),
            return_unit(),
        ],
        vec![Constant::Integer(1)],
        vec!["root", "test.fpas", "answer"],
        1,
    );
    image.globals = vec![GlobalInfo {
        name: StringId::new(2),
        mutable: false,
    }];
    let error = execute(image.verify().expect("global image must verify"))
        .expect_err("second immutable store must fail");
    assert_eq!(error.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
    assert!(error.message.contains("assigned more than once"));
}

#[test]
fn missing_indexes_keep_existing_runtime_codes() {
    let array = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abc(Opcode::MakeArray, 2, 0, 1),
            abc(Opcode::IndexGet, 3, 2, 1),
            return_unit(),
        ],
        vec![Constant::Integer(4), Constant::Integer(9)],
        vec!["root", "test.fpas"],
        4,
    );
    let error = execute(array).expect_err("out-of-bounds array read must fail");
    assert_eq!(error.code, RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS);

    let dictionary = verified(
        vec![
            abx(Opcode::LoadConstant, 0, 0),
            abx(Opcode::LoadConstant, 1, 1),
            abx(Opcode::LoadConstant, 2, 2),
            abc(Opcode::MakeDictionary, 3, 0, 1),
            abc(Opcode::IndexGet, 4, 3, 2),
            return_unit(),
        ],
        vec![
            Constant::String(StringId::new(2)),
            Constant::Integer(4),
            Constant::String(StringId::new(3)),
        ],
        vec!["root", "test.fpas", "present", "missing"],
        5,
    );
    let error = execute(dictionary).expect_err("missing dictionary key must fail");
    assert_eq!(error.code, RUNTIME_DICT_KEY_NOT_FOUND);
}

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
        }],
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
