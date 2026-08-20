//! JSONL mapping for variant discovery and complete construction.

use serde_json::{Map, Value, json};

use super::{DebugEngine, DebugStatus};
use crate::evaluation::{parse_debug_assignment_target, parse_debug_expression};
use crate::jsonl::encode::{invalid_state, missing_argument, optional_u64_argument};
use crate::jsonl::protocol::{failure, session_error, success};

impl DebugEngine {
    pub(super) fn describe_variant(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let request = match parse_target(request_id, command, arguments) {
            Ok(request) => request,
            Err(response) => return vec![response],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.describe_variant_with_limits(
            &request.target,
            request.frame_id,
            request.limits,
        ) {
            Ok(description) => vec![success(
                request_id,
                command,
                json!({
                    "target": request.target_source,
                    "type_name": description.type_name,
                    "variants": description.variants.iter().map(|variant| {
                        json!({
                            "name": variant.name,
                            "fields": variant.fields.iter().map(|field| {
                                json!({
                                    "name": field.name,
                                    "type_name": field.type_name
                                })
                            }).collect::<Vec<_>>()
                        })
                    }).collect::<Vec<_>>()
                }),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }

    pub(super) fn construct_variant(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        if self.status != DebugStatus::Stopped {
            return vec![invalid_state(request_id, command, self.status)];
        }
        let request = match parse_target(request_id, command, arguments) {
            Ok(request) => request,
            Err(response) => return vec![response],
        };
        let Some(variant) = arguments.get("variant").and_then(Value::as_str) else {
            return vec![missing_argument(request_id, command, "variant")];
        };
        let fields = match parse_fields(request_id, command, arguments, request.limits) {
            Ok(fields) => fields,
            Err(response) => return vec![response],
        };
        let Some(session) = self.actor.session_mut() else {
            return vec![invalid_state(request_id, command, self.status)];
        };
        match session.construct_variant_with_limits(
            &request.target,
            variant,
            &fields,
            request.frame_id,
            request.limits,
        ) {
            Ok(result) => vec![success(
                request_id,
                command,
                json!({
                    "result": result.value.value,
                    "type_name": result.value.type_name,
                    "variables_reference": result.value.variables_reference,
                    "named_variables": result.value.named_variables,
                    "indexed_variables": result.value.indexed_variables,
                    "variant": result.variant
                }),
            )],
            Err(error) => vec![session_error(request_id, command, error)],
        }
    }
}

struct TargetRequest {
    target: fpas_vm::DebugAssignmentTarget,
    target_source: String,
    frame_id: Option<u64>,
    limits: fpas_vm::DebugEvaluationLimits,
}

fn parse_target(
    request_id: u64,
    command: &str,
    arguments: &Map<String, Value>,
) -> Result<TargetRequest, Value> {
    let Some(target_source) = arguments.get("target").and_then(Value::as_str) else {
        return Err(missing_argument(request_id, command, "target"));
    };
    let frame_id = optional_u64_argument(request_id, command, arguments, "frame_id")?;
    let limits = fpas_vm::DebugEvaluationLimits::default();
    let target = parse_debug_assignment_target(target_source, limits)
        .map_err(|error| parse_error(request_id, command, error))?;
    Ok(TargetRequest {
        target,
        target_source: target_source.to_string(),
        frame_id,
        limits,
    })
}

fn parse_fields(
    request_id: u64,
    command: &str,
    arguments: &Map<String, Value>,
    limits: fpas_vm::DebugEvaluationLimits,
) -> Result<Vec<(String, fpas_vm::DebugExpression)>, Value> {
    match arguments.get("fields") {
        None | Some(Value::Null) => Err(missing_argument(request_id, command, "fields")),
        Some(Value::Object(fields)) => fields
            .iter()
            .map(|(name, value)| {
                let Some(source) = value.as_str() else {
                    return Err(failure(
                        request_id,
                        command,
                        "invalid_request",
                        format!(
                            "Command `{command}` field `{name}` must be an FPAS expression string."
                        ),
                        "Pass one expression string for every declared field.",
                    ));
                };
                parse_debug_expression(source, limits)
                    .map(|expression| (name.clone(), expression))
                    .map_err(|error| parse_error(request_id, command, error))
            })
            .collect(),
        Some(_) => Err(failure(
            request_id,
            command,
            "invalid_request",
            format!("Command `{command}` argument `fields` must be an object."),
            "Pass `{}` for a fieldless variant, or a name-to-expression object for payload fields.",
        )),
    }
}

fn parse_error(
    request_id: u64,
    command: &str,
    error: crate::evaluation::EvaluationParseError,
) -> Value {
    json!({
        "type": "response",
        "request_id": request_id,
        "command": command,
        "success": false,
        "error": {
            "code": error.code,
            "message": error.message,
            "help": error.hint,
            "offset": error.offset,
            "length": error.length
        }
    })
}
