//! JSONL parsing for mutation, return, and storage operations.

use serde_json::{Map, Value};

use super::args::{
    missing, optional_expression_string, optional_u64, require_string_typed, required_string,
    required_u64,
};
use crate::engine::{DebugOp, EngineFailure};

pub(super) fn parse_frame_return(
    command: &str,
    arguments: &Map<String, Value>,
) -> Result<DebugOp, EngineFailure> {
    Ok(DebugOp::FrameReturn {
        frame_id: required_u64(command, arguments, "frame_id")?,
        expression: optional_expression_string(arguments, "frame.return")?,
    })
}

pub(super) fn parse_instruction_set(
    command: &str,
    arguments: &Map<String, Value>,
) -> Result<DebugOp, EngineFailure> {
    let frame_id = match arguments.get("frame_id") {
        None | Some(Value::Null) => None,
        Some(value) => Some(value.as_u64().ok_or_else(|| missing(command, "frame_id"))?),
    };
    let instruction = match arguments.get("instruction") {
        None | Some(Value::Null) => None,
        Some(value) => Some(
            value
                .as_u64()
                .and_then(|value| u32::try_from(value).ok())
                .ok_or_else(|| missing(command, "instruction"))?,
        ),
    };
    Ok(DebugOp::InstructionSet {
        frame_id,
        instruction,
    })
}

pub(super) fn parse_task_result_replace(
    command: &str,
    arguments: &Map<String, Value>,
) -> Result<DebugOp, EngineFailure> {
    Ok(DebugOp::TaskResultReplace {
        task_id: required_u64(command, arguments, "task_id")?,
        expression: optional_expression_string(arguments, "task.result.replace")?,
        frame_id: optional_u64(command, arguments, "frame_id")?,
    })
}

pub(super) fn parse_variant_construct(
    command: &str,
    arguments: &Map<String, Value>,
) -> Result<DebugOp, EngineFailure> {
    let fields = match arguments.get("fields") {
        None | Some(Value::Null) => return Err(missing(command, "fields")),
        Some(Value::Object(fields)) => fields
            .iter()
            .map(|(name, value)| {
                let Some(source) = value.as_str() else {
                    return Err(EngineFailure::new(
                        "invalid_request",
                        format!(
                            "Command `{command}` field `{name}` must be an FPAS expression string."
                        ),
                        "Pass one expression string for every declared field.",
                    ));
                };
                Ok((name.clone(), source.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?,
        Some(_) => {
            return Err(EngineFailure::new(
                "invalid_request",
                format!("Command `{command}` argument `fields` must be an object."),
                "Pass `{}` for a fieldless variant, or a name-to-expression object for payload fields.",
            ));
        }
    };
    Ok(DebugOp::VariantConstruct {
        target: required_string(command, arguments, "target")?,
        variant: required_string(command, arguments, "variant")?,
        fields,
        frame_id: optional_u64(command, arguments, "frame_id")?,
    })
}

pub(super) fn parse_storage(
    command: &str,
    arguments: &Map<String, Value>,
) -> Result<DebugOp, EngineFailure> {
    let extra = arguments.keys().find(|key| {
        !matches!(
            key.as_str(),
            "frame_id" | "target" | "initializer" | "expression"
        )
    });
    if let Some(name) = extra {
        return Err(EngineFailure::new(
            "invalid_request",
            format!("Command `{command}` does not accept argument `{name}`."),
            "Pass only `frame_id`, `target`, `initializer`, and `expression`.",
        ));
    }
    Ok(DebugOp::StorageInitialize {
        target: require_string_typed(command, arguments, "target")?,
        initializer: require_string_typed(command, arguments, "initializer")?,
        expression: require_string_typed(command, arguments, "expression")?,
        frame_id: optional_u64(command, arguments, "frame_id")?,
    })
}
