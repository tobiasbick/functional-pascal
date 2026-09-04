use std::sync::Arc;

use fpas_bytecode::{
    CodeRange, DebugType, DebugTypeId, EnumLayout, EnumTypeId, EnumVariant, EnumVariantId,
    Executable, FunctionFlags, FunctionId, FunctionInfo, Instruction, InstructionAddress, Opcode,
    ReturnConvention, RuntimeEnumLayout, SharedEnum, SourceId, SourceMap, SourceRun, StringId,
    StringTable, Value,
};

use super::{MutationPath, PayloadError, resolve};

fn image(debug_types: Vec<DebugType>, enum_variants: Vec<EnumVariant>) -> Executable {
    Executable {
        code: vec![
            Instruction::abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0).expect("return"),
        ],
        functions: vec![FunctionInfo {
            name: StringId::new(0),
            code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(1)),
            arity: 0,
            capture_count: 0,
            register_count: 0,
            return_convention: ReturnConvention::Unit,
            flags: FunctionFlags::default(),
            debug: fpas_bytecode::FunctionDebugInfo::default(),
        }],
        constants: Vec::new(),
        strings: StringTable::new(vec!["root".to_string(), "test.fpas".to_string()]),
        globals: Vec::new(),
        records: Vec::new(),
        enums: vec![EnumLayout {
            name: StringId::new(0),
        }],
        enum_variants,
        debug_types,
        source_map: SourceMap {
            sources: vec![StringId::new(1)],
            runs: vec![SourceRun {
                instruction_start: InstructionAddress::new(0),
                source: SourceId::new(0),
                line: 1,
                column: 1,
            }],
        },
        entry: FunctionId::new(0),
    }
}

fn choice(variant_id: u16, variant: &str, fields: &[&str], values: Vec<Value>) -> Value {
    Value::Enum(SharedEnum::new(
        Arc::new(RuntimeEnumLayout {
            enumeration: EnumTypeId::new(0),
            variant_id: EnumVariantId::new(variant_id),
            type_name: "Choice".to_string(),
            variant: variant.to_string(),
            fields: fields.iter().map(|field| (*field).to_string()).collect(),
        }),
        values,
    ))
}

fn payload_types() -> Executable {
    image(
        vec![
            DebugType::Integer,
            DebugType::String,
            DebugType::Enum(EnumTypeId::new(0)),
            DebugType::Result {
                ok: DebugTypeId::new(0),
                error: DebugTypeId::new(1),
            },
            DebugType::Option(DebugTypeId::new(0)),
        ],
        vec![
            EnumVariant {
                owner: EnumTypeId::new(0),
                name: StringId::new(0),
                fields: vec![StringId::new(0)],
                field_types: vec![DebugTypeId::new(0)],
            },
            EnumVariant {
                owner: EnumTypeId::new(0),
                name: StringId::new(0),
                fields: vec![StringId::new(0), StringId::new(0)],
                field_types: vec![DebugTypeId::new(0), DebugTypeId::new(0)],
            },
        ],
    )
}

#[test]
fn resolves_each_result_branch_some_and_enum_field_positions() {
    let executable = payload_types();
    let ok = resolve(
        &executable,
        DebugTypeId::new(3),
        &Value::result_ok(Value::Integer(1)),
        "value",
    )
    .expect("ok payload");
    assert_eq!(ok, (MutationPath::ResultOk, DebugTypeId::new(0)));

    let error = resolve(
        &executable,
        DebugTypeId::new(3),
        &Value::result_error(Value::Str("e".into())),
        "VaLuE",
    )
    .expect("error payload");
    assert_eq!(error, (MutationPath::ResultError, DebugTypeId::new(1)));

    let some = resolve(
        &executable,
        DebugTypeId::new(4),
        &Value::option_some(Value::Integer(2)),
        "value",
    )
    .expect("some payload");
    assert_eq!(some, (MutationPath::OptionSome, DebugTypeId::new(0)));

    let count = resolve(
        &executable,
        DebugTypeId::new(2),
        &choice(0, "Count", &["Value"], vec![Value::Integer(1)]),
        "value",
    )
    .expect("count field");
    assert_eq!(
        count,
        (
            MutationPath::EnumField {
                variant: EnumVariantId::new(0),
                index: 0
            },
            DebugTypeId::new(0)
        )
    );

    let right = resolve(
        &executable,
        DebugTypeId::new(2),
        &choice(
            1,
            "Pair",
            &["Left", "Right"],
            vec![Value::Integer(1), Value::Integer(2)],
        ),
        "right",
    )
    .expect("pair field");
    assert_eq!(
        right,
        (
            MutationPath::EnumField {
                variant: EnumVariantId::new(1),
                index: 1
            },
            DebugTypeId::new(0)
        )
    );
}

#[test]
fn rejects_none_unknown_names_and_inconsistent_metadata() {
    let executable = payload_types();
    assert!(matches!(
        resolve(
            &executable,
            DebugTypeId::new(4),
            &Value::OptionNone,
            "value"
        ),
        Err(PayloadError::UnsupportedActive { active }) if active == "Option.None"
    ));
    assert!(matches!(
        resolve(
            &executable,
            DebugTypeId::new(3),
            &Value::result_ok(Value::Integer(1)),
            "count"
        ),
        Err(PayloadError::UnknownField { active }) if active == "Result.Ok"
    ));
    assert!(matches!(
        resolve(
            &executable,
            DebugTypeId::new(2),
            &choice(0, "Count", &["Value"], vec![Value::Integer(1)]),
            "left"
        ),
        Err(PayloadError::UnknownField { active }) if active == "Choice.Count"
    ));
    assert!(matches!(
        resolve(
            &executable,
            DebugTypeId::new(0),
            &Value::result_ok(Value::Integer(1)),
            "value"
        ),
        Err(PayloadError::UnavailableMetadata { .. })
    ));

    let missing_variant = image(
        vec![DebugType::Integer, DebugType::Enum(EnumTypeId::new(0))],
        Vec::new(),
    );
    assert!(matches!(
        resolve(
            &missing_variant,
            DebugTypeId::new(1),
            &choice(0, "Count", &["Value"], vec![Value::Integer(1)]),
            "value"
        ),
        Err(PayloadError::UnavailableMetadata { .. })
    ));

    let owner_mismatch = image(
        vec![DebugType::Integer, DebugType::Enum(EnumTypeId::new(0))],
        vec![EnumVariant {
            owner: EnumTypeId::new(1),
            name: StringId::new(0),
            fields: vec![StringId::new(0)],
            field_types: vec![DebugTypeId::new(0)],
        }],
    );
    assert!(matches!(
        resolve(
            &owner_mismatch,
            DebugTypeId::new(1),
            &choice(0, "Count", &["Value"], vec![Value::Integer(1)]),
            "value"
        ),
        Err(PayloadError::UnavailableMetadata { .. })
    ));

    let short_metadata = image(
        vec![DebugType::Integer, DebugType::Enum(EnumTypeId::new(0))],
        vec![EnumVariant {
            owner: EnumTypeId::new(0),
            name: StringId::new(0),
            fields: Vec::new(),
            field_types: Vec::new(),
        }],
    );
    assert!(matches!(
        resolve(
            &short_metadata,
            DebugTypeId::new(1),
            &choice(0, "Count", &["Value"], vec![Value::Integer(1)]),
            "value"
        ),
        Err(PayloadError::UnavailableMetadata { .. })
    ));

    let count_mismatch = image(
        vec![DebugType::Integer, DebugType::Enum(EnumTypeId::new(0))],
        vec![EnumVariant {
            owner: EnumTypeId::new(0),
            name: StringId::new(0),
            fields: vec![StringId::new(0), StringId::new(0)],
            field_types: vec![DebugTypeId::new(0), DebugTypeId::new(0)],
        }],
    );
    assert!(matches!(
        resolve(
            &count_mismatch,
            DebugTypeId::new(1),
            &choice(0, "Count", &["Value"], vec![Value::Integer(1)]),
            "value"
        ),
        Err(PayloadError::UnavailableMetadata { .. })
    ));
}
