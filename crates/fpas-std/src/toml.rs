//! Runtime implementations for `Std.Toml.*` intrinsics.
//!
//! **Documentation:** `docs/pascal/std/text/toml.md`

use crate::error::{StdError, std_runtime_error};
use crate::intrinsic_args::{IntrinsicCall, pop_string, pop_value};
use crate::limits::MAX_TOML_DEPTH;
use crate::std_units::std_symbols as s;
use fpas_bytecode::{Intrinsic, SourceLocation, TomlIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;
use toml::Value as TomlValue;
use toml::map::Map;
use toml::value::Datetime;

fn toml_variant(
    call: &IntrinsicCall<'_>,
    variant: &str,
    fields: Vec<Value>,
    location: SourceLocation,
) -> Result<Value, StdError> {
    call.enumeration(s::STD_TOML_VALUE, variant, fields, location)
}

fn toml_depth_exceeded_message() -> String {
    format!("TOML nesting exceeds maximum depth of {MAX_TOML_DEPTH}")
}

fn toml_to_fpas_at_depth(
    call: &IntrinsicCall<'_>,
    value: TomlValue,
    depth: usize,
    location: SourceLocation,
) -> Result<Value, StdError> {
    if depth > MAX_TOML_DEPTH {
        return Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            toml_depth_exceeded_message(),
            format!("Keep TOML trees at most {MAX_TOML_DEPTH} levels deep."),
            location,
        ));
    }

    match value {
        TomlValue::String(value) => {
            toml_variant(call, "String", vec![Value::Str(value.into())], location)
        }
        TomlValue::Integer(value) => {
            toml_variant(call, "Integer", vec![Value::Integer(value)], location)
        }
        TomlValue::Float(value) => toml_variant(call, "Float", vec![Value::Real(value)], location),
        TomlValue::Boolean(value) => {
            toml_variant(call, "Boolean", vec![Value::Boolean(value)], location)
        }
        TomlValue::Datetime(value) => toml_variant(
            call,
            "Datetime",
            vec![Value::Str(value.to_string().into())],
            location,
        ),
        TomlValue::Array(items) => items
            .into_iter()
            .map(|item| toml_to_fpas_at_depth(call, item, depth + 1, location))
            .collect::<Result<Vec<_>, _>>()
            .and_then(|items| {
                toml_variant(call, "Array", vec![Value::Array(items.into())], location)
            }),
        TomlValue::Table(fields) => fields
            .into_iter()
            .map(|(key, value)| {
                toml_to_fpas_at_depth(call, value, depth + 1, location)
                    .map(|value| (Value::Str(key.into()), value))
            })
            .collect::<Result<Vec<_>, _>>()
            .and_then(|fields| toml_variant(call, "Table", vec![Value::dict(fields)], location)),
    }
}

fn toml_to_fpas(
    call: &IntrinsicCall<'_>,
    value: TomlValue,
    location: SourceLocation,
) -> Result<Value, StdError> {
    toml_to_fpas_at_depth(call, value, 1, location)
}

fn expected_toml_value_error(value: &Value, location: SourceLocation) -> StdError {
    std_runtime_error(
        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        format!("expected Std.Toml.TomlValue, got {}", value.type_name()),
        "Pass a value constructed with Std.Toml.TomlValue.* or returned by Std.Toml.Parse.",
        location,
    )
}

fn expect_one_field(
    variant: &str,
    fields: Vec<Value>,
    location: SourceLocation,
) -> Result<Value, StdError> {
    if let [field] = fields.as_slice() {
        return Ok(field.clone());
    }
    Err(std_runtime_error(
        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        format!("Std.Toml.TomlValue.{variant} expects exactly one runtime field"),
        "Use the documented Std.Toml.TomlValue constructors without manually altering enum payloads.",
        location,
    ))
}

fn fpas_to_toml_at_depth(
    value: Value,
    location: SourceLocation,
    depth: usize,
) -> Result<TomlValue, StdError> {
    if depth > MAX_TOML_DEPTH {
        return Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            toml_depth_exceeded_message(),
            format!("Keep TOML trees at most {MAX_TOML_DEPTH} levels deep."),
            location,
        ));
    }

    let (type_name, variant, fields) = match value {
        Value::Enum(value) => {
            let body = value.body();
            (
                body.layout.type_name.clone(),
                body.layout.variant.clone(),
                body.values.clone(),
            )
        }
        value => return Err(expected_toml_value_error(&value, location)),
    };

    if !type_name.eq_ignore_ascii_case(s::STD_TOML_VALUE) {
        return Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("expected Std.Toml.TomlValue, got enum {type_name}"),
            "Pass a value constructed with Std.Toml.TomlValue.* or returned by Std.Toml.Parse.",
            location,
        ));
    }

    match variant.as_str() {
        "String" => match expect_one_field("String", fields, location)? {
            Value::Str(value) => Ok(TomlValue::String(value.into())),
            other => variant_field_error("String", "string", &other, location),
        },
        "Integer" => match expect_one_field("Integer", fields, location)? {
            Value::Integer(value) => Ok(TomlValue::Integer(value)),
            other => variant_field_error("Integer", "integer", &other, location),
        },
        "Float" => match expect_one_field("Float", fields, location)? {
            Value::Real(value) => Ok(TomlValue::Float(value)),
            Value::Integer(value) => Ok(TomlValue::Float(value as f64)),
            other => variant_field_error("Float", "real", &other, location),
        },
        "Boolean" => match expect_one_field("Boolean", fields, location)? {
            Value::Boolean(value) => Ok(TomlValue::Boolean(value)),
            other => variant_field_error("Boolean", "boolean", &other, location),
        },
        "Datetime" => match expect_one_field("Datetime", fields, location)? {
            Value::Str(value) => value.parse::<Datetime>().map(TomlValue::Datetime).map_err(|error| {
                std_runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!("Std.Toml.TomlValue.Datetime is invalid: {error}"),
                    "Pass a TOML date, time, or date-time string such as `1979-05-27T07:32:00Z`.",
                    location,
                )
            }),
            other => variant_field_error("Datetime", "string", &other, location),
        },
        "Array" => match expect_one_field("Array", fields, location)? {
            Value::Array(items) => items
                .into_iter()
                .map(|item| fpas_to_toml_at_depth(item, location, depth + 1))
                .collect::<Result<Vec<_>, _>>()
                .map(TomlValue::Array),
            other => variant_field_error("Array", "array", &other, location),
        },
        "Table" => match expect_one_field("Table", fields, location)? {
            Value::Dict(fields) => {
                let mut table = Map::new();
                for (key, value) in fields {
                    let Value::Str(key) = key else {
                        return Err(std_runtime_error(
                            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                            format!(
                                "Std.Toml.TomlValue.Table expects string keys, got {}",
                                key.type_name()
                            ),
                            "Use `dict of string to TomlValue` for TOML tables.",
                            location,
                        ));
                    };
                    table.insert(
                        key.into(),
                        fpas_to_toml_at_depth(value, location, depth + 1)?,
                    );
                }
                Ok(TomlValue::Table(table))
            }
            other => variant_field_error("Table", "dict", &other, location),
        },
        other => Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("unknown Std.Toml.TomlValue variant `{other}`"),
            "Use String, Integer, Float, Boolean, Datetime, Array, or Table.",
            location,
        )),
    }
}

fn variant_field_error(
    variant: &str,
    expected: &str,
    actual: &Value,
    location: SourceLocation,
) -> Result<TomlValue, StdError> {
    Err(std_runtime_error(
        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        format!(
            "Std.Toml.TomlValue.{variant} expects {expected}, got {}",
            actual.type_name()
        ),
        format!("Construct {variant} with a {expected} value."),
        location,
    ))
}

fn fpas_to_toml(value: Value, location: SourceLocation) -> Result<TomlValue, StdError> {
    fpas_to_toml_at_depth(value, location, 1)
}

/// Executes `Std.Toml` intrinsics.
///
/// **Documentation:** `docs/pascal/std/text/toml.md`
pub(crate) fn run(
    intrinsic: Intrinsic,
    call: &mut IntrinsicCall<'_>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Toml(TomlIntrinsic::Parse) => {
            let text = pop_string(pop_value(call, location)?, location)?;
            match toml::from_str::<TomlValue>(&text).map_err(|error| error.to_string()) {
                Ok(value) => match toml_to_fpas(call, value, location) {
                    Ok(value) => call.push(Value::ResultOk(Box::new(value))),
                    Err(error) => call.push(Value::ResultError(Box::new(Value::Str(
                        error.message.into(),
                    )))),
                },
                Err(message) => call.push(Value::ResultError(Box::new(Value::Str(message.into())))),
            }
        }
        Intrinsic::Toml(TomlIntrinsic::Stringify) => {
            let value = pop_value(call, location)?;
            let toml = fpas_to_toml(value.clone(), location)?;
            let text = toml::to_string(&toml).map_err(|error| {
                std_runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!("cannot stringify TOML value: {error}"),
                    "Pass a TOML table or a value that can be encoded in TOML.",
                    location,
                )
            })?;
            call.push(Value::Str(text.into()));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    fn call() -> IntrinsicCall<'static> {
        IntrinsicCall::new(&[], &crate::intrinsics::TEST_AGGREGATES)
    }

    fn test_variant(variant: &str, fields: Vec<Value>) -> Value {
        toml_variant(&call(), variant, fields, loc()).expect("test aggregate layout")
    }

    fn test_toml_to_fpas_at_depth(value: TomlValue, depth: usize) -> Result<Value, StdError> {
        toml_to_fpas_at_depth(&call(), value, depth, loc())
    }

    #[test]
    fn parse_preserves_all_toml_value_kinds() {
        let text = r#"
title = "TOML"
integer = 42
float = 1.5
boolean = true
when = 1979-05-27T07:32:00Z
items = ["a", 2]
[nested]
value = "ok"
"#;
        let mut stack = vec![Value::Str(text.into())];

        crate::execute_test_intrinsic(Intrinsic::Toml(TomlIntrinsic::Parse), &mut stack, loc())
            .unwrap();

        assert!(matches!(stack.as_slice(), [Value::ResultOk(_)]));
        let Value::ResultOk(value) = &stack[0] else {
            panic!("expected a parsed TOML value");
        };
        let Value::Enum(value) = value.as_ref() else {
            panic!("expected a TomlValue enum");
        };
        assert_eq!(value.body().layout.variant, "Table");
    }

    #[test]
    fn parse_returns_error_for_invalid_toml() {
        let mut stack = vec![Value::Str("answer = [1,".into())];

        crate::execute_test_intrinsic(Intrinsic::Toml(TomlIntrinsic::Parse), &mut stack, loc())
            .unwrap();

        assert!(matches!(stack.as_slice(), [Value::ResultError(_)]));
    }

    #[test]
    fn toml_to_fpas_accepts_container_at_depth_limit() {
        let value = TomlValue::Array(vec![TomlValue::String("ok".into())]);

        assert!(test_toml_to_fpas_at_depth(value, MAX_TOML_DEPTH - 1).is_ok());
    }

    #[test]
    fn toml_to_fpas_rejects_container_child_beyond_depth_limit() {
        let value = TomlValue::Array(vec![TomlValue::String("too deep".into())]);

        assert!(test_toml_to_fpas_at_depth(value, MAX_TOML_DEPTH).is_err());
    }

    #[test]
    fn fpas_to_toml_accepts_container_at_depth_limit() {
        let value = test_variant(
            "Array",
            vec![Value::Array(
                vec![test_variant("String", vec![Value::Str("ok".into())])].into(),
            )],
        );

        assert!(fpas_to_toml_at_depth(value, loc(), MAX_TOML_DEPTH - 1).is_ok());
    }

    #[test]
    fn fpas_to_toml_rejects_container_child_beyond_depth_limit() {
        let value = test_variant(
            "Array",
            vec![Value::Array(
                vec![test_variant("String", vec![Value::Str("too deep".into())])].into(),
            )],
        );

        let error = fpas_to_toml_at_depth(value, loc(), MAX_TOML_DEPTH).unwrap_err();

        assert_eq!(error.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
    }

    #[test]
    fn stringify_rejects_invalid_datetime() {
        let value = test_variant("Datetime", vec![Value::Str("not-a-date".into())]);

        let error = fpas_to_toml(value, loc()).unwrap_err();

        assert_eq!(error.code, RUNTIME_VM_OPERAND_TYPE_MISMATCH);
        assert!(error.message.contains("Datetime is invalid"));
    }

    #[test]
    fn stringify_round_trips_array_of_tables() {
        let mut stack = vec![Value::Str(
            "[[products]]\nname = \"Hammer\"\n[[products]]\nname = \"Nail\"\n".into(),
        )];
        crate::execute_test_intrinsic(Intrinsic::Toml(TomlIntrinsic::Parse), &mut stack, loc())
            .unwrap();
        let Value::ResultOk(value) = stack.pop().expect("parse result") else {
            panic!("expected a parsed TOML value");
        };
        let mut stringify_stack = vec![*value];

        crate::execute_test_intrinsic(
            Intrinsic::Toml(TomlIntrinsic::Stringify),
            &mut stringify_stack,
            loc(),
        )
        .unwrap();

        let Value::Str(text) = stringify_stack.pop().expect("stringified TOML") else {
            panic!("expected TOML text");
        };
        assert!(text.contains("[[products]]"));
    }
}
