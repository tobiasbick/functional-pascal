//! Shared runtime and metadata resolution for writable payload descendants.

use fpas_bytecode::{DebugType, DebugTypeId, EnumValue, Executable, Value};

use super::MutationPath;

/// Structured failure while resolving one payload child name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::vm::debug) enum PayloadError {
    /// The name is absent from the currently active payload.
    UnknownField {
        /// Active variant or wrapper label used in diagnostics.
        active: String,
    },
    /// The live value has no writable payload for this selector.
    UnsupportedActive {
        /// Active variant or wrapper label used in diagnostics.
        active: String,
    },
    /// Portable metadata is missing or inconsistent with the live value.
    UnavailableMetadata {
        /// Bounded explanation of the metadata failure.
        detail: String,
    },
}

/// Resolves one payload field name to a guarded path component and declared type.
pub(in crate::vm::debug) fn resolve(
    executable: &Executable,
    expected: DebugTypeId,
    current: &Value,
    name: &str,
) -> Result<(MutationPath, DebugTypeId), PayloadError> {
    match current {
        Value::Enum(enumeration) => enum_field(executable, expected, enumeration.body(), name),
        Value::ResultOk(_) => wrapper_field(
            executable,
            expected,
            name,
            WrapperKind::ResultOk,
            MutationPath::ResultOk,
        ),
        Value::ResultError(_) => wrapper_field(
            executable,
            expected,
            name,
            WrapperKind::ResultError,
            MutationPath::ResultError,
        ),
        Value::OptionSome(_) => wrapper_field(
            executable,
            expected,
            name,
            WrapperKind::OptionSome,
            MutationPath::OptionSome,
        ),
        Value::OptionNone => Err(PayloadError::UnsupportedActive {
            active: active_label(current),
        }),
        _ => Err(PayloadError::UnsupportedActive {
            active: active_label(current),
        }),
    }
}

/// Human-readable label for the currently active payload-bearing value.
pub(in crate::vm::debug) fn active_label(value: &Value) -> String {
    match value {
        Value::Enum(enumeration) => {
            let layout = &enumeration.body().layout;
            format!("{}.{}", layout.type_name, layout.variant)
        }
        other => other.type_name().to_string(),
    }
}

#[derive(Clone, Copy)]
enum WrapperKind {
    ResultOk,
    ResultError,
    OptionSome,
}

fn enum_field(
    executable: &Executable,
    expected: DebugTypeId,
    body: &EnumValue,
    name: &str,
) -> Result<(MutationPath, DebugTypeId), PayloadError> {
    let active = format!("{}.{}", body.layout.type_name, body.layout.variant);
    let Some(index) = body
        .layout
        .fields
        .iter()
        .position(|field| field.eq_ignore_ascii_case(name))
    else {
        return Err(PayloadError::UnknownField { active });
    };
    let Some(DebugType::Enum(enumeration)) = debug_type(executable, expected) else {
        return Err(PayloadError::UnavailableMetadata {
            detail: format!("declared type for `{active}` is not an enum layout"),
        });
    };
    if body.layout.enumeration != *enumeration {
        return Err(PayloadError::UnavailableMetadata {
            detail: format!("enum `{active}` does not match its declared type owner"),
        });
    }
    let Some(variant) = executable
        .enum_variants
        .get(usize::from(body.layout.variant_id.get()))
    else {
        return Err(PayloadError::UnavailableMetadata {
            detail: format!("enum variant metadata for `{active}` is unavailable"),
        });
    };
    if variant.owner != body.layout.enumeration || variant.owner != *enumeration {
        return Err(PayloadError::UnavailableMetadata {
            detail: format!("enum variant `{active}` is not owned by its declared type"),
        });
    }
    if body.values.len() != variant.field_types.len() {
        return Err(PayloadError::UnavailableMetadata {
            detail: format!("enum variant `{active}` field count does not match metadata"),
        });
    }
    let Some(field_type) = variant.field_types.get(index).copied() else {
        return Err(PayloadError::UnavailableMetadata {
            detail: format!("enum field `{name}` on `{active}` is out of range"),
        });
    };
    require_declared_type(executable, field_type)?;
    Ok((
        MutationPath::EnumField {
            variant: body.layout.variant_id,
            index,
        },
        field_type,
    ))
}

fn wrapper_field(
    executable: &Executable,
    expected: DebugTypeId,
    name: &str,
    kind: WrapperKind,
    component: MutationPath,
) -> Result<(MutationPath, DebugTypeId), PayloadError> {
    let active = match kind {
        WrapperKind::ResultOk => "Result.Ok",
        WrapperKind::ResultError => "Result.Error",
        WrapperKind::OptionSome => "Option.Some",
    };
    if !name.eq_ignore_ascii_case("value") {
        return Err(PayloadError::UnknownField {
            active: active.to_string(),
        });
    }
    let declared = match (kind, debug_type(executable, expected)) {
        (WrapperKind::ResultOk, Some(DebugType::Result { ok, .. })) => *ok,
        (WrapperKind::ResultError, Some(DebugType::Result { error, .. })) => *error,
        (WrapperKind::OptionSome, Some(DebugType::Option(inner))) => *inner,
        (kind, Some(_)) => {
            return Err(PayloadError::UnavailableMetadata {
                detail: format!(
                    "declared type for `{active}` does not match the live {} payload",
                    match kind {
                        WrapperKind::ResultOk => "Result.Ok",
                        WrapperKind::ResultError => "Result.Error",
                        WrapperKind::OptionSome => "Option.Some",
                    }
                ),
            });
        }
        (_, None) => {
            return Err(PayloadError::UnavailableMetadata {
                detail: format!("declared type for `{active}` is unavailable"),
            });
        }
    };
    require_declared_type(executable, declared)?;
    Ok((component, declared))
}

fn debug_type(executable: &Executable, expected: DebugTypeId) -> Option<&DebugType> {
    executable.debug_types.get(expected.get() as usize)
}

fn require_declared_type(executable: &Executable, ty: DebugTypeId) -> Result<(), PayloadError> {
    if debug_type(executable, ty).is_some() {
        Ok(())
    } else {
        Err(PayloadError::UnavailableMetadata {
            detail: format!("payload field type #{} is unavailable", ty.get()),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fpas_bytecode::{
        CodeRange, DebugType, DebugTypeId, EnumLayout, EnumTypeId, EnumVariant, EnumVariantId,
        Executable, FunctionFlags, FunctionId, FunctionInfo, Instruction, InstructionAddress,
        Opcode, ReturnConvention, RuntimeEnumLayout, SharedEnum, SourceId, SourceMap, SourceRun,
        StringId, StringTable, Value,
    };

    use super::{MutationPath, PayloadError, resolve};

    fn image(debug_types: Vec<DebugType>, enum_variants: Vec<EnumVariant>) -> Executable {
        Executable {
            code: vec![
                Instruction::abc(Opcode::Return, fpas_bytecode::NO_REGISTER, 0, 0, 0)
                    .expect("return"),
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
            &Value::ResultOk(Box::new(Value::Integer(1))),
            "value",
        )
        .expect("ok payload");
        assert_eq!(ok, (MutationPath::ResultOk, DebugTypeId::new(0)));

        let error = resolve(
            &executable,
            DebugTypeId::new(3),
            &Value::ResultError(Box::new(Value::Str("e".into()))),
            "VaLuE",
        )
        .expect("error payload");
        assert_eq!(error, (MutationPath::ResultError, DebugTypeId::new(1)));

        let some = resolve(
            &executable,
            DebugTypeId::new(4),
            &Value::OptionSome(Box::new(Value::Integer(2))),
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
                &Value::ResultOk(Box::new(Value::Integer(1))),
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
                &Value::ResultOk(Box::new(Value::Integer(1))),
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
}
