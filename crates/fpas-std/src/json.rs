//! Runtime implementations for `Std.Json.*` intrinsics.
//!
//! **Documentation:** `docs/pascal/std/json.md`

use crate::error::{StdError, std_runtime_error};
use crate::intrinsic_args::{pop_string, pop_value};
use crate::std_units::std_symbols as s;
use fpas_bytecode::{Intrinsic, JsonIntrinsic, SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;
use serde_json::{Map, Number, Value as JsonValue};

fn json_variant(variant: &str, fields: Vec<Value>) -> Value {
    Value::Enum {
        type_name: s::STD_JSON_VALUE.into(),
        variant: variant.into(),
        fields,
    }
}

fn json_to_fpas(value: JsonValue) -> Result<Value, String> {
    match value {
        JsonValue::Null => Ok(json_variant("Null", Vec::new())),
        JsonValue::Bool(value) => Ok(json_variant("Bool", vec![Value::Boolean(value)])),
        JsonValue::Number(number) => {
            let Some(value) = number.as_f64() else {
                return Err("JSON number is outside the FPAS real range".into());
            };
            Ok(json_variant("Number", vec![Value::Real(value)]))
        }
        JsonValue::String(value) => Ok(json_variant("String", vec![Value::Str(value)])),
        JsonValue::Array(items) => items
            .into_iter()
            .map(json_to_fpas)
            .collect::<Result<Vec<_>, _>>()
            .map(|items| json_variant("Array", vec![Value::Array(items)])),
        JsonValue::Object(fields) => fields
            .into_iter()
            .map(|(key, value)| json_to_fpas(value).map(|value| (Value::Str(key), value)))
            .collect::<Result<Vec<_>, _>>()
            .map(|fields| json_variant("Object", vec![Value::Dict(fields)])),
    }
}

fn expected_json_value_error(value: &Value, location: SourceLocation) -> StdError {
    std_runtime_error(
        RUNTIME_VM_OPERAND_TYPE_MISMATCH,
        format!("expected Std.Json.JsonValue, got {}", value.type_name()),
        "Pass a value constructed with Std.Json.JsonValue.* or returned by Std.Json.Parse.",
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
        format!("Std.Json.JsonValue.{variant} expects exactly one runtime field"),
        "Use the documented Std.Json.JsonValue constructors without manually altering enum payloads.",
        location,
    ))
}

fn fpas_to_json(value: Value, location: SourceLocation) -> Result<JsonValue, StdError> {
    let Value::Enum {
        type_name,
        variant,
        fields,
    } = value
    else {
        return Err(expected_json_value_error(&value, location));
    };

    if !type_name.eq_ignore_ascii_case(s::STD_JSON_VALUE) {
        return Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("expected Std.Json.JsonValue, got enum {type_name}"),
            "Pass a value constructed with Std.Json.JsonValue.* or returned by Std.Json.Parse.",
            location,
        ));
    }

    match variant.as_str() {
        "Null" => {
            if fields.is_empty() {
                Ok(JsonValue::Null)
            } else {
                Err(std_runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    "Std.Json.JsonValue.Null must not carry runtime fields",
                    "Use Std.Json.JsonValue.Null without arguments.",
                    location,
                ))
            }
        }
        "Bool" => match expect_one_field("Bool", fields, location)? {
            Value::Boolean(value) => Ok(JsonValue::Bool(value)),
            other => Err(std_runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!(
                    "Std.Json.JsonValue.Bool expects boolean, got {}",
                    other.type_name()
                ),
                "Construct booleans with Std.Json.JsonValue.Bool(true) or Bool(false).",
                location,
            )),
        },
        "Number" => match expect_one_field("Number", fields, location)? {
            Value::Real(value) => match Number::from_f64(value) {
                Some(number) => Ok(JsonValue::Number(number)),
                None => Err(std_runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    "Std.Json.JsonValue.Number cannot stringify NaN or infinity",
                    "Pass a finite real value.",
                    location,
                )),
            },
            Value::Integer(value) => Ok(JsonValue::Number(Number::from(value))),
            other => Err(std_runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!(
                    "Std.Json.JsonValue.Number expects real, got {}",
                    other.type_name()
                ),
                "Construct numbers with Std.Json.JsonValue.Number(1.5).",
                location,
            )),
        },
        "String" => match expect_one_field("String", fields, location)? {
            Value::Str(value) => Ok(JsonValue::String(value)),
            Value::Char(value) => Ok(JsonValue::String(value.to_string())),
            other => Err(std_runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!(
                    "Std.Json.JsonValue.String expects string, got {}",
                    other.type_name()
                ),
                "Construct strings with Std.Json.JsonValue.String('text').",
                location,
            )),
        },
        "Array" => match expect_one_field("Array", fields, location)? {
            Value::Array(items) => items
                .into_iter()
                .map(|item| fpas_to_json(item, location))
                .collect::<Result<Vec<_>, _>>()
                .map(JsonValue::Array),
            other => Err(std_runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!(
                    "Std.Json.JsonValue.Array expects array, got {}",
                    other.type_name()
                ),
                "Construct arrays with Std.Json.JsonValue.Array([Item1, Item2]).",
                location,
            )),
        },
        "Object" => match expect_one_field("Object", fields, location)? {
            Value::Dict(fields) => {
                let mut object = Map::new();
                for (key, value) in fields {
                    let key = match key {
                        Value::Str(key) => key,
                        Value::Char(key) => key.to_string(),
                        other => {
                            return Err(std_runtime_error(
                                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                                format!(
                                    "Std.Json.JsonValue.Object expects string keys, got {}",
                                    other.type_name()
                                ),
                                "Use `dict of string to Std.Json.JsonValue` for JSON objects.",
                                location,
                            ));
                        }
                    };
                    object.insert(key, fpas_to_json(value, location)?);
                }
                Ok(JsonValue::Object(object))
            }
            other => Err(std_runtime_error(
                RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                format!(
                    "Std.Json.JsonValue.Object expects dict, got {}",
                    other.type_name()
                ),
                "Construct objects with Std.Json.JsonValue.Object(['key': Value]).",
                location,
            )),
        },
        other => Err(std_runtime_error(
            RUNTIME_VM_OPERAND_TYPE_MISMATCH,
            format!("unknown Std.Json.JsonValue variant `{other}`"),
            "Use one of Null, Bool, Number, String, Array, or Object.",
            location,
        )),
    }
}

pub(crate) fn run(
    intrinsic: Intrinsic,
    stack: &mut Vec<Value>,
    location: SourceLocation,
) -> Result<Option<()>, StdError> {
    match intrinsic {
        Intrinsic::Json(JsonIntrinsic::Parse) => {
            let text = pop_string(pop_value(stack, location)?, location)?;
            match serde_json::from_str::<JsonValue>(&text).map_err(|err| err.to_string()) {
                Ok(value) => match json_to_fpas(value) {
                    Ok(value) => stack.push(Value::ResultOk(Box::new(value))),
                    Err(message) => stack.push(Value::ResultError(Box::new(Value::Str(message)))),
                },
                Err(message) => stack.push(Value::ResultError(Box::new(Value::Str(message)))),
            }
        }
        Intrinsic::Json(JsonIntrinsic::Stringify) => {
            let value = pop_value(stack, location)?;
            let json = fpas_to_json(value, location)?;
            stack.push(Value::Str(json.to_string()));
        }
        _ => return Ok(None),
    }
    Ok(Some(()))
}
