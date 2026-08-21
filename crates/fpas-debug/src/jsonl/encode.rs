//! JSONL response and event body encoding.

use serde_json::{Value, json};

pub(crate) fn initialize_body(execution: fpas_vm::DebugExecutionLimits, hot_reload: bool) -> Value {
    let inspection = fpas_vm::DebugInspectionLimits::default();
    let evaluation = fpas_vm::DebugEvaluationLimits::default();
    let breakpoints = fpas_vm::DebugBreakpointLimits::default();
    let mut capabilities = json!({
        "source_breakpoints": true,
        "function_breakpoints": true,
        "runtime_failure_filters": true,
        "pause": true,
        "continue": true,
        "step_into": true,
        "step_over": true,
        "step_out": true,
        "stack_pagination": true,
        "scope_inspection": true,
        "variable_pagination": true,
        "aggregate_expansion": true,
        "structured_output": true,
        "attach": false,
        "non_stop": false,
        "task_threads": true,
        "task_pause": true,
        "task_cancel": true,
        "task_create": false,
        "task_restart": false,
        "evaluate": true,
        "evaluate_calls": true,
        "set_variable": true,
        "set_expression": true,
        "dictionary_insert": true,
        "dictionary_remove": true,
        "dictionary_replace_key": true,
        "array_insert": true,
        "array_remove": true,
        "string_replace_character": true,
        "frame_return": true,
        "frame_restart": true,
        "instruction_set": false,
        "task_result_replacement": true,
        "variant_describe": true,
        "variant_construct": true,
        "storage_initialize": true,
        "conditional_breakpoints": true,
        "hit_conditions": true,
        "logpoints": true,
        "reverse_execution": false
    });
    if let Value::Object(capabilities) = &mut capabilities {
        capabilities.insert("live_input".into(), json!(true));
        capabilities.insert("live_terminal".into(), json!(false));
        capabilities.insert("data_breakpoints".into(), json!(true));
        capabilities.insert("data_breakpoint_access".into(), json!(["write", "change"]));
        capabilities.insert("location_describe".into(), json!(true));
        capabilities.insert("breakpoint_assign".into(), json!(true));
        capabilities.insert("record_replay".into(), json!(false));
        capabilities.insert("recording_describe".into(), json!(true));
        capabilities.insert("recording_capture".into(), json!(true));
        capabilities.insert("recording_disk".into(), json!(false));
        capabilities.insert("hot_reload".into(), json!(hot_reload));
        capabilities.insert("reload_classify".into(), json!(true));
        capabilities.insert("reload_rollback".into(), json!(true));
    }
    json!({
        "protocol": "fpas-debug-jsonl",
        "version": 2,
        "capabilities": capabilities,
        "limits": {
            "stack_frames": inspection.max_frames,
            "variables": inspection.max_children,
            "value_depth": inspection.max_depth,
            "string_characters": inspection.max_string_chars,
            "retained_handles": inspection.max_handles,
            "breakpoints": breakpoints.max_breakpoints,
            "function_breakpoint_bindings": breakpoints.max_function_bindings,
            "function_name_bytes": breakpoints.max_function_name_bytes,
            "runtime_failure_filters": crate::breakpoints::MAX_RUNTIME_FAILURE_FILTERS,
            "expression_bytes": evaluation.max_expression_bytes,
            "expression_depth": evaluation.max_depth,
            "expression_operations": evaluation.max_operations,
            "expression_traversals": evaluation.max_traversals,
            "expression_output_bytes": evaluation.max_output_bytes,
            "evaluation_calls": evaluation.max_calls,
            "evaluation_call_depth": evaluation.max_call_depth,
            "evaluation_call_instructions": evaluation.max_call_instructions,
            "evaluation_detached_values": evaluation.max_detached_values,
            "evaluation_call_timeout_milliseconds": evaluation.call_timeout.as_millis(),
            "captured_output_bytes": execution.max_output_bytes,
            "debuggee_input_bytes": execution.max_input_bytes,
            "recording_events": fpas_vm::MAX_RECORDING_EVENTS,
            "recording_snapshots": 0,
            "instructions": execution.max_instructions,
            "timeout_milliseconds": execution.timeout.as_millis()
        }
    })
}

pub(crate) fn breakpoint_body(breakpoint: &fpas_vm::BoundBreakpoint) -> Value {
    json!({"breakpoint_id":breakpoint.id,"verified":breakpoint.is_verified(),"requested":{"source":breakpoint.requested.source,"line":breakpoint.requested.line,"column":breakpoint.requested.column},"location":breakpoint.location.as_ref().map(location_body),"message":(!breakpoint.is_verified()).then_some("No executable sequence point exists on the requested line.")})
}

pub(crate) fn function_breakpoint_body(breakpoint: &fpas_vm::BoundFunctionBreakpoint) -> Value {
    let message = if breakpoint.functions.is_empty() {
        Some("No executable function metadata matches the requested selector.".to_string())
    } else if breakpoint.instructions.is_empty() {
        Some("Matching functions have no executable entry sequence point.".to_string())
    } else if breakpoint.functions.len() > 1 {
        Some(format!(
            "Bound to {} exact functions in executable order.",
            breakpoint.functions.len()
        ))
    } else {
        None
    };
    json!({
        "breakpoint_id": breakpoint.id,
        "verified": breakpoint.is_verified(),
        "requested": {"name": breakpoint.requested.name},
        "matched_functions": breakpoint.functions.iter().map(|function| function.get()).collect::<Vec<_>>(),
        "match_count": breakpoint.functions.len(),
        "locations": breakpoint.locations.iter().map(location_body).collect::<Vec<_>>(),
        "message": message
    })
}

pub(crate) fn task_body(task: &fpas_vm::DebugTask) -> Value {
    json!({
        "task_id": task.id,
        "name": task.name,
        "state": task.state.as_str(),
        "inspectable": task.inspectable,
        "paused": task.paused
    })
}

pub(crate) fn data_breakpoint_body(breakpoint: &fpas_vm::BoundDataBreakpoint) -> Value {
    json!({
        "breakpoint_id": breakpoint.id,
        "verified": breakpoint.is_verified(),
        "requested": {
            "identity": identity_body(breakpoint.requested.identity),
            "access": breakpoint.requested.access.as_str()
        },
        "message": breakpoint.message
    })
}

pub(crate) fn identity_body(identity: fpas_vm::DebugDataLocationIdentity) -> Value {
    match identity {
        fpas_vm::DebugDataLocationIdentity::Global { index } => json!({"index": index}),
        fpas_vm::DebugDataLocationIdentity::FrameRegister {
            task_id,
            function,
            register,
        } => json!({
            "task_id": task_id,
            "function": function,
            "register": register
        }),
    }
}

fn location_body(location: &fpas_vm::SourceLocation) -> Value {
    json!({"source":location.source,"line":location.line,"column":location.column})
}

pub(crate) fn frame_body(frame: &fpas_vm::DebugFrame) -> Value {
    json!({"frame_id":frame.id,"name":frame.name,"location":frame.location.as_ref().map(location_body),"depth":frame.depth})
}

pub(crate) fn scope_body(scope: &fpas_vm::DebugScope) -> Value {
    json!({"name":scope.name,"kind":format!("{:?}",scope.kind).to_ascii_lowercase(),"variables_reference":scope.variables_reference,"named_variables":scope.named_variables,"expensive":scope.expensive})
}

pub(crate) fn variable_body(variable: &fpas_vm::DebugVariable) -> Value {
    json!({"name":variable.name,"value":variable.value,"type_name":variable.type_name,"variables_reference":variable.variables_reference,"named_variables":variable.named_variables,"indexed_variables":variable.indexed_variables,"presentation_hint":variable.presentation_hint})
}
