//! DAP request routing onto JSONL core commands and adapter-local handlers.

use serde_json::{Value, json};

use super::DapServer;
use crate::jsonl::ServerStatus;

impl DapServer {
    pub(super) fn dispatch_request(
        &mut self,
        request_seq: u64,
        command: &str,
        arguments: &Value,
    ) -> Vec<Value> {
        match command {
            "initialize" => self.initialize(request_seq, arguments),
            "launch" => {
                self.stop_on_entry = arguments
                    .get("stopOnEntry")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                vec![self.success(request_seq, command, json!({}))]
            }
            "attach" => vec![self.failure(
                request_seq,
                command,
                "DAP attach is unsupported; launch a Functional Pascal program instead.",
            )],
            "stepBack" | "reverseContinue" => vec![self.failure(
                request_seq,
                command,
                "Reverse execution is unsupported; the debugger cannot step backward or replay.",
            )],
            "disassemble" | "readMemory" | "writeMemory" => vec![self.failure(
                request_seq,
                command,
                "Native memory and disassembly are unsupported; the debugger inspects FPAS bytecode.",
            )],
            "setDataBreakpoints" => self.set_data_breakpoints(request_seq, arguments),
            "dataBreakpointInfo" => self.data_breakpoint_info(request_seq, arguments),
            "setBreakpoints" => self.set_source_breakpoints(request_seq, arguments),
            "setFunctionBreakpoints" => {
                self.set_function_breakpoints(request_seq, arguments)
            }
            "setExceptionBreakpoints" => {
                self.set_exception_breakpoints(request_seq, arguments)
            }
            "configurationDone" => self.core_request(
                request_seq,
                command,
                "launch",
                json!({"stop_on_entry": self.stop_on_entry}),
            ),
            "threads" if self.core.status() == ServerStatus::Running => {
                let body = self.threads.active_threads();
                vec![self.success(request_seq, command, body)]
            }
            "threads" => self.core_request(request_seq, command, "tasks", json!({})),
            "stackTrace" => match self.task_id(arguments, "threadId") {
                Ok(task_id) => self.core_request(
                    request_seq,
                    command,
                    "stack",
                    json!({
                        "task_id": task_id,
                        "start": arguments.get("startFrame").and_then(Value::as_u64).unwrap_or(0),
                        "count": arguments.get("levels").and_then(Value::as_u64).unwrap_or(64)
                    }),
                ),
                Err(message) => vec![self.failure(request_seq, command, &message)],
            },
            "scopes" => self.core_request(
                request_seq,
                command,
                "scopes",
                json!({"frame_id": arguments.get("frameId").cloned().unwrap_or(Value::Null)}),
            ),
            "variables" => self.core_request(
                request_seq,
                command,
                "variables",
                json!({
                    "variables_reference": arguments.get("variablesReference").cloned().unwrap_or(Value::Null),
                    "start": arguments.get("start").cloned().unwrap_or(json!(0)),
                    "count": arguments.get("count").cloned().unwrap_or(json!(100))
                }),
            ),
            "evaluate" => self.evaluate(request_seq, command, arguments),
            "setVariable" => self.set_variable(request_seq, command, arguments),
            "setExpression" => self.set_expression(request_seq, command, arguments),
            "fpas/dictionaryInsert" => self.insert_dictionary(request_seq, command, arguments),
            "fpas/dictionaryRemove" => self.remove_dictionary(request_seq, command, arguments),
            "fpas/dictionaryReplaceKey" => {
                self.replace_dictionary_key(request_seq, command, arguments)
            }
            "fpas/arrayInsert" => self.insert_array(request_seq, command, arguments),
            "fpas/arrayRemove" => self.remove_array(request_seq, command, arguments),
            "fpas/stringReplaceCharacter" => {
                self.replace_string_character(request_seq, command, arguments)
            }
            "fpas/forceReturn" => self.force_return(request_seq, command, arguments),
            "restartFrame" => self.restart_frame(request_seq, command, arguments),
            "goto" | "gotoTargets" => self.set_instruction(request_seq, command, arguments),
            "fpas/replaceTaskResult" => {
                self.replace_completed_task_result(request_seq, command, arguments)
            }
            "fpas/pauseTask" => self.pause_task(request_seq, command, arguments),
            "fpas/resumeTask" => self.resume_task(request_seq, command, arguments),
            "fpas/cancelTask" => self.cancel_task(request_seq, command, arguments),
            "fpas/createTask" => self.create_task(request_seq, command),
            "fpas/restartTask" => self.restart_task(request_seq, command, arguments),
            "fpas/input" => self.push_debuggee_input(request_seq, command, arguments),
            "fpas/eof" => self.signal_debuggee_eof(request_seq, command),
            "fpas/cancelInput" => self.cancel_debuggee_input(request_seq, command),
            "fpas/variantDescribe" => self.describe_variant(request_seq, command, arguments),
            "fpas/locationDescribe" => self.describe_location(request_seq, command, arguments),
            "fpas/recordingDescribe" => self.describe_recording(request_seq, command),
            "fpas/variantConstruct" => self.construct_variant(request_seq, command, arguments),
            "fpas/initializeStorage" => self.initialize_storage(request_seq, command, arguments),
            "cancel" => self.core_request(
                request_seq,
                command,
                "evaluate.cancel",
                json!({"request_id": arguments.get("requestId")}),
            ),
            "continue" if self.runtime_failed => {
                self.runtime_failed = false;
                self.core_request(request_seq, command, "disconnect", json!({}))
            }
            "continue" => self.core_request(request_seq, command, "continue", json!({})),
            "pause" => self.core_request(request_seq, command, "pause", json!({})),
            "next" => self.step_request(request_seq, command, "step_over", arguments),
            "stepIn" => self.step_request(request_seq, command, "step_into", arguments),
            "stepOut" => self.step_request(request_seq, command, "step_out", arguments),
            "disconnect" => self.core_request(request_seq, command, "disconnect", json!({})),
            "source" => self.source(request_seq, command, arguments),
            _ => vec![self.failure(
                request_seq,
                command,
                &format!("DAP request `{command}` is unsupported by the FPAS debugger."),
            )],
        }
    }
}
