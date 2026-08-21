//! DAP request routing onto typed debug engine operations.

use serde_json::{Value, json};

use super::DapServer;
use super::args;
use crate::engine::{DebugOp, DebugStatus};

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
            "setFunctionBreakpoints" => self.set_function_breakpoints(request_seq, arguments),
            "setExceptionBreakpoints" => self.set_exception_breakpoints(request_seq, arguments),
            "configurationDone" => self.core_request(
                request_seq,
                command,
                DebugOp::Launch {
                    stop_on_entry: self.stop_on_entry,
                },
            ),
            "threads" if self.core.status() == DebugStatus::Running => {
                let body = self.threads.active_threads();
                vec![self.success(request_seq, command, body)]
            }
            "threads" => self.core_request(
                request_seq,
                command,
                DebugOp::Tasks {
                    start: 0,
                    count: 64,
                },
            ),
            "stackTrace" => match self.task_id(arguments, "threadId") {
                Ok(task_id) => {
                    let count = args::page_count(
                        arguments.get("levels"),
                        fpas_vm::DebugInspectionLimits::default().max_frames,
                    );
                    let start = arguments
                        .get("startFrame")
                        .and_then(Value::as_u64)
                        .and_then(|value| usize::try_from(value).ok())
                        .unwrap_or(0);
                    self.core_request(
                        request_seq,
                        command,
                        DebugOp::Stack {
                            start,
                            count,
                            task_id: Some(task_id),
                        },
                    )
                }
                Err(message) => vec![self.failure(request_seq, command, &message)],
            },
            "scopes" => match args::required_u64(arguments, "frameId") {
                Ok(frame_id) => {
                    self.core_request(request_seq, command, DebugOp::Scopes { frame_id })
                }
                Err(message) => vec![self.failure(request_seq, command, &message)],
            },
            "variables" => match args::required_u64(arguments, "variablesReference") {
                Ok(variables_reference) => {
                    let count = args::page_count(
                        arguments.get("count"),
                        fpas_vm::DebugInspectionLimits::default().max_children,
                    );
                    let start = arguments
                        .get("start")
                        .and_then(Value::as_u64)
                        .and_then(|value| usize::try_from(value).ok())
                        .unwrap_or(0);
                    self.core_request(
                        request_seq,
                        command,
                        DebugOp::Variables {
                            variables_reference,
                            start,
                            count,
                        },
                    )
                }
                Err(message) => vec![self.failure(request_seq, command, &message)],
            },
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
            "fpas/record" => self.start_recording(request_seq, command),
            "fpas/reloadClassify" => self.classify_live_image(request_seq, command),
            "fpas/reload" => self.replace_current_live_image(request_seq, command),
            "fpas/reloadRollback" => self.rollback_live_image(request_seq, command),
            "fpas/variantConstruct" => self.construct_variant(request_seq, command, arguments),
            "fpas/initializeStorage" => self.initialize_storage(request_seq, command, arguments),
            "cancel" => self.core_request(request_seq, command, DebugOp::EvaluateCancel),
            "continue" if self.runtime_failed => {
                self.runtime_failed = false;
                self.core_request(request_seq, command, DebugOp::Disconnect)
            },
            "continue" => self.core_request(request_seq, command, DebugOp::Continue),
            "pause" => self.core_request(request_seq, command, DebugOp::Pause),
            "next" => self.step_request(request_seq, command, arguments, |task_id| {
                DebugOp::StepOver {
                    task_id: Some(task_id),
                }
            }),
            "stepIn" => self.step_request(request_seq, command, arguments, |task_id| {
                DebugOp::StepInto {
                    task_id: Some(task_id),
                }
            }),
            "stepOut" => self.step_request(request_seq, command, arguments, |task_id| {
                DebugOp::StepOut {
                    task_id: Some(task_id),
                }
            }),
            "disconnect" => self.core_request(request_seq, command, DebugOp::Disconnect),
            "source" => self.source(request_seq, command, arguments),
            _ => vec![self.failure(
                request_seq,
                command,
                &format!("DAP request `{command}` is unsupported by the FPAS debugger."),
            )],
        }
    }
}
