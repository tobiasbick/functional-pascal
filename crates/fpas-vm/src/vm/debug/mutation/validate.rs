//! Bounded validation of replacement values against portable debugger types.

use fpas_bytecode::{DebugType, DebugTypeId, Executable, Value};

use super::super::types::{DebugErrorKind, DebugSessionError};

pub(super) fn value(
    executable: &Executable,
    expected: DebugTypeId,
    value: &Value,
    max_depth: usize,
) -> Result<(), DebugSessionError> {
    validate(executable, expected, value, max_depth, 0)
}

fn validate(
    executable: &Executable,
    expected: DebugTypeId,
    value: &Value,
    max_depth: usize,
    depth: usize,
) -> Result<(), DebugSessionError> {
    if depth > max_depth {
        return Err(type_error(
            expected,
            value,
            "replacement exceeds the validation depth",
        ));
    }
    let ty = executable
        .debug_types
        .get(expected.get() as usize)
        .ok_or_else(|| type_error(expected, value, "declared debugger type is unavailable"))?;
    match (ty, value) {
        (DebugType::Unit, Value::Unit)
        | (DebugType::Boolean, Value::Boolean(_))
        | (DebugType::Integer, Value::Integer(_))
        | (DebugType::Real, Value::Real(_))
        | (DebugType::String, Value::Str(_)) => Ok(()),
        (DebugType::Dynamic, value) => validate_dynamic(value, max_depth, depth),
        (DebugType::Array(inner), Value::Array(values)) => values
            .iter()
            .try_for_each(|value| validate(executable, *inner, value, max_depth, depth + 1)),
        (DebugType::Dictionary { key, value }, Value::Dict(entries)) => {
            for (entry_key, entry_value) in entries {
                validate(executable, *key, entry_key, max_depth, depth + 1)?;
                validate(executable, *value, entry_value, max_depth, depth + 1)?;
            }
            Ok(())
        }
        (DebugType::Result { ok, .. }, Value::ResultOk(inner)) => {
            validate(executable, *ok, inner, max_depth, depth + 1)
        }
        (DebugType::Result { error, .. }, Value::ResultError(inner)) => {
            validate(executable, *error, inner, max_depth, depth + 1)
        }
        (DebugType::Option(inner), Value::OptionSome(value)) => {
            validate(executable, *inner, value, max_depth, depth + 1)
        }
        (DebugType::Option(_), Value::OptionNone) => Ok(()),
        (DebugType::Record(record), Value::Record(runtime))
            if runtime.body().layout.record == *record =>
        {
            let layout = &executable.records[usize::from(record.get())];
            for (field, value) in layout.fields.iter().zip(&runtime.body().values) {
                validate(executable, field.ty, value, max_depth, depth + 1)?;
            }
            Ok(())
        }
        (DebugType::Enum(enumeration), Value::Enum(runtime))
            if runtime.body().layout.enumeration == *enumeration =>
        {
            let body = runtime.body();
            let variant = executable
                .enum_variants
                .get(body.layout.variant_id.get() as usize)
                .ok_or_else(|| type_error(expected, value, "enum variant is unavailable"))?;
            if variant.owner != *enumeration {
                return Err(type_error(
                    expected,
                    value,
                    "enum variant owner does not match the declared type",
                ));
            }
            let owner_name = executable
                .enums
                .get(usize::from(enumeration.get()))
                .and_then(|layout| executable.strings.get(layout.name));
            let variant_name = executable.strings.get(variant.name);
            if owner_name.is_some_and(|name| !name.eq_ignore_ascii_case(&body.layout.type_name))
                || variant_name.is_some_and(|name| !name.eq_ignore_ascii_case(&body.layout.variant))
            {
                return Err(type_error(
                    expected,
                    value,
                    "enum runtime layout names do not match executable metadata",
                ));
            }
            if variant.fields.len() != variant.field_types.len()
                || variant.fields.len() != body.layout.fields.len()
                || variant.fields.len() != body.values.len()
            {
                return Err(type_error(
                    expected,
                    value,
                    "enum variant field count does not match",
                ));
            }
            for (index, field_id) in variant.fields.iter().enumerate() {
                let Some(expected_name) = executable.strings.get(*field_id) else {
                    return Err(type_error(
                        expected,
                        value,
                        "enum variant field name is unavailable",
                    ));
                };
                if !body
                    .layout
                    .fields
                    .get(index)
                    .is_some_and(|name| name.eq_ignore_ascii_case(expected_name))
                {
                    return Err(type_error(
                        expected,
                        value,
                        "enum variant field layout does not match",
                    ));
                }
            }
            for (field_type, field_value) in variant.field_types.iter().zip(&body.values) {
                validate(executable, *field_type, field_value, max_depth, depth + 1)?;
            }
            Ok(())
        }
        (DebugType::Function { .. }, Value::Function(function)) if depth == 0 => {
            super::function_value::validate_root(function, max_depth, 65_536)
        }
        (DebugType::Function { .. } | DebugType::Cell(_) | DebugType::Task(_), _) => {
            Err(type_error(
                expected,
                value,
                "this target type is not assignable by the debugger",
            ))
        }
        _ => Err(type_error(
            expected,
            value,
            "replacement type does not match",
        )),
    }
}

fn validate_dynamic(
    value: &Value,
    max_depth: usize,
    depth: usize,
) -> Result<(), DebugSessionError> {
    if depth > max_depth {
        return Err(dynamic_error(
            value,
            "replacement exceeds the validation depth",
        ));
    }
    match value {
        Value::Function(_) | Value::Cell(_) | Value::Task(_) | Value::OpaqueHandle(_) => {
            Err(dynamic_error(
                value,
                "dynamic assignment rejects live or opaque runtime values",
            ))
        }
        Value::Array(values) => values
            .iter()
            .try_for_each(|value| validate_dynamic(value, max_depth, depth + 1)),
        Value::Dict(entries) => {
            for (key, value) in entries {
                validate_dynamic(key, max_depth, depth + 1)?;
                validate_dynamic(value, max_depth, depth + 1)?;
            }
            Ok(())
        }
        Value::Record(record) => record
            .body()
            .values
            .iter()
            .try_for_each(|value| validate_dynamic(value, max_depth, depth + 1)),
        Value::Enum(enumeration) => enumeration
            .body()
            .values
            .iter()
            .try_for_each(|value| validate_dynamic(value, max_depth, depth + 1)),
        Value::ResultOk(value) | Value::ResultError(value) | Value::OptionSome(value) => {
            validate_dynamic(value, max_depth, depth + 1)
        }
        Value::Integer(_)
        | Value::Real(_)
        | Value::Boolean(_)
        | Value::Str(_)
        | Value::Unit
        | Value::OptionNone => Ok(()),
    }
}

fn type_error(expected: DebugTypeId, value: &Value, detail: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: format!(
            "debug replacement value `{}` does not match type #{}: {detail}",
            value.type_name(),
            expected.get()
        ),
        hint: "Use an expression whose complete value matches the declared FPAS type.".to_string(),
    }
}

fn dynamic_error(value: &Value, detail: &str) -> DebugSessionError {
    DebugSessionError {
        kind: DebugErrorKind::VariableValueType,
        message: format!("debug replacement value `{}` is rejected: {detail}", value.type_name()),
        hint: "Use a detached scalar or aggregate value without functions, tasks, cells, or opaque handles."
            .to_string(),
    }
}
