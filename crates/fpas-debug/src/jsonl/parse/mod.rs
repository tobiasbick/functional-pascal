//! JSONL argument parsing onto typed debug engine operations.

mod args;
mod breakpoints;
mod structure;

use serde_json::{Map, Value};

use self::args::{index_argument, optional_u64, parse_string_array, required_string, required_u64};
use self::breakpoints::{parse_breakpoint_set, parse_data_breakpoints, parse_function_breakpoints};
use self::structure::{
    parse_frame_return, parse_instruction_set, parse_storage, parse_task_result_replace,
    parse_variant_construct,
};
use crate::engine::{DebugOp, EngineFailure};

/// Parse one JSONL command name and argument object into a typed engine operation.
pub(crate) fn parse_op(
    command: &str,
    arguments: &Map<String, Value>,
) -> Result<DebugOp, EngineFailure> {
    match command {
        "initialize" => parse_initialize(arguments),
        "launch" => Ok(DebugOp::Launch {
            stop_on_entry: arguments
                .get("stop_on_entry")
                .and_then(Value::as_bool)
                .unwrap_or(true),
        }),
        "pause" => Ok(DebugOp::Pause),
        "continue" => Ok(DebugOp::Continue),
        "step_into" => Ok(DebugOp::StepInto {
            task_id: optional_u64(command, arguments, "task_id")?,
        }),
        "step_over" => Ok(DebugOp::StepOver {
            task_id: optional_u64(command, arguments, "task_id")?,
        }),
        "step_out" => Ok(DebugOp::StepOut {
            task_id: optional_u64(command, arguments, "task_id")?,
        }),
        "attach" => Ok(DebugOp::Attach),
        "step_back" => Ok(DebugOp::StepBack),
        "reverse_continue" => Ok(DebugOp::ReverseContinue),
        "replay" => Ok(DebugOp::Replay),
        "reload" => Ok(DebugOp::Reload),
        "image.replace" => Ok(DebugOp::ImageReplace),
        "image.rollback" => Ok(DebugOp::ImageRollback),
        "reload.classify" => Ok(DebugOp::ReloadClassify),
        "record" => Ok(DebugOp::Record),
        "data_breakpoint.set" => Ok(DebugOp::DataBreakpointSet),
        "data_breakpoints.replace" => Ok(DebugOp::DataBreakpointsReplace {
            breakpoints: parse_data_breakpoints(command, arguments)?,
        }),
        "breakpoint.set" => parse_breakpoint_set(command, arguments),
        "breakpoint.clear" => Ok(DebugOp::BreakpointClear {
            breakpoint_id: required_u64(command, arguments, "breakpoint_id")?,
        }),
        "function_breakpoints.replace" => Ok(DebugOp::FunctionBreakpointsReplace {
            breakpoints: parse_function_breakpoints(command, arguments)?,
        }),
        "runtime_failures.replace" => Ok(DebugOp::RuntimeFailuresReplace {
            filters: parse_string_array(command, arguments, "filters")?,
        }),
        "tasks" => Ok(DebugOp::Tasks {
            start: index_argument(arguments, "start", 0),
            count: index_argument(arguments, "count", 64),
        }),
        "task.pause" => Ok(DebugOp::TaskPause {
            task_id: required_u64(command, arguments, "task_id")?,
        }),
        "task.resume" => Ok(DebugOp::TaskResume {
            task_id: required_u64(command, arguments, "task_id")?,
        }),
        "task.cancel" => Ok(DebugOp::TaskCancel {
            task_id: required_u64(command, arguments, "task_id")?,
        }),
        "task.create" => Ok(DebugOp::TaskCreate),
        "task.restart" => parse_task_restart(command, arguments),
        "io.input" => Ok(DebugOp::IoInput {
            text: required_string(command, arguments, "text")?,
        }),
        "io.eof" => Ok(DebugOp::IoEof),
        "io.cancel" => Ok(DebugOp::IoCancel),
        "stack" => Ok(DebugOp::Stack {
            start: index_argument(arguments, "start", 0),
            count: index_argument(arguments, "count", 64),
            task_id: optional_u64(command, arguments, "task_id")?,
        }),
        "scopes" => Ok(DebugOp::Scopes {
            frame_id: required_u64(command, arguments, "frame_id")?,
        }),
        "variables" => Ok(DebugOp::Variables {
            variables_reference: required_u64(command, arguments, "variables_reference")?,
            start: index_argument(arguments, "start", 0),
            count: index_argument(arguments, "count", 100),
        }),
        "evaluate" => Ok(DebugOp::Evaluate {
            expression: required_string(command, arguments, "expression")?,
            frame_id: arguments.get("frame_id").and_then(Value::as_u64),
            async_eval: arguments
                .get("async")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        }),
        "variable.set" => Ok(DebugOp::VariableSet {
            variables_reference: required_u64(command, arguments, "variables_reference")?,
            name: required_string(command, arguments, "name")?,
            expression: required_string(command, arguments, "expression")?,
        }),
        "expression.set" => Ok(DebugOp::ExpressionSet {
            target: required_string(command, arguments, "target")?,
            expression: required_string(command, arguments, "expression")?,
            frame_id: optional_u64(command, arguments, "frame_id")?,
        }),
        "dictionary.insert" => Ok(DebugOp::DictionaryInsert {
            target: required_string(command, arguments, "target")?,
            key: required_string(command, arguments, "key")?,
            expression: required_string(command, arguments, "expression")?,
            frame_id: optional_u64(command, arguments, "frame_id")?,
        }),
        "dictionary.remove" => Ok(DebugOp::DictionaryRemove {
            target: required_string(command, arguments, "target")?,
            key: required_string(command, arguments, "key")?,
            frame_id: optional_u64(command, arguments, "frame_id")?,
        }),
        "dictionary.replace_key" => Ok(DebugOp::DictionaryReplaceKey {
            target: required_string(command, arguments, "target")?,
            key: required_string(command, arguments, "key")?,
            new_key: required_string(command, arguments, "new_key")?,
            frame_id: optional_u64(command, arguments, "frame_id")?,
        }),
        "array.insert" => Ok(DebugOp::ArrayInsert {
            target: required_string(command, arguments, "target")?,
            index: required_string(command, arguments, "index")?,
            expression: required_string(command, arguments, "expression")?,
            frame_id: optional_u64(command, arguments, "frame_id")?,
        }),
        "array.remove" => Ok(DebugOp::ArrayRemove {
            target: required_string(command, arguments, "target")?,
            index: required_string(command, arguments, "index")?,
            frame_id: optional_u64(command, arguments, "frame_id")?,
        }),
        "string.replace_character" => Ok(DebugOp::StringReplaceCharacter {
            target: required_string(command, arguments, "target")?,
            index: required_string(command, arguments, "index")?,
            expression: required_string(command, arguments, "expression")?,
            frame_id: optional_u64(command, arguments, "frame_id")?,
        }),
        "frame.return" => parse_frame_return(command, arguments),
        "frame.restart" => Ok(DebugOp::FrameRestart {
            frame_id: required_u64(command, arguments, "frame_id")?,
        }),
        "instruction.set" => parse_instruction_set(command, arguments),
        "location.describe" => Ok(DebugOp::LocationDescribe {
            variables_reference: required_u64(command, arguments, "variables_reference")?,
            name: required_string(command, arguments, "name")?,
        }),
        "recording.describe" => Ok(DebugOp::RecordingDescribe),
        "task.result.replace" => parse_task_result_replace(command, arguments),
        "variant.describe" => Ok(DebugOp::VariantDescribe {
            target: required_string(command, arguments, "target")?,
            frame_id: optional_u64(command, arguments, "frame_id")?,
        }),
        "variant.construct" => parse_variant_construct(command, arguments),
        "storage.initialize" => parse_storage(command, arguments),
        "evaluate.cancel" => Ok(DebugOp::EvaluateCancel),
        "disconnect" => Ok(DebugOp::Disconnect),
        other => Ok(DebugOp::Unknown(other.to_string())),
    }
}

fn parse_initialize(arguments: &Map<String, Value>) -> Result<DebugOp, EngineFailure> {
    let version = arguments
        .get("version")
        .and_then(Value::as_u64)
        .unwrap_or(2);
    if version != 2 {
        return Err(EngineFailure::new(
            "unsupported_protocol_version",
            format!("Protocol version {version} is unsupported."),
            "Request version 2.",
        ));
    }
    Ok(DebugOp::Initialize)
}

fn parse_task_restart(
    command: &str,
    arguments: &Map<String, Value>,
) -> Result<DebugOp, EngineFailure> {
    let task_id = match arguments.get("task_id") {
        None | Some(Value::Null) => None,
        Some(value) => Some(
            value
                .as_u64()
                .ok_or_else(|| args::missing(command, "task_id"))?,
        ),
    };
    Ok(DebugOp::TaskRestart { task_id })
}
