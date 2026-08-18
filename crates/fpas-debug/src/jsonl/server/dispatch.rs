//! JSONL command routing for one protocol V2 server.

use serde_json::{Map, Value};

use super::super::actor::ResumeCommand;
use super::super::protocol::failure;
use super::JsonlServer;

impl JsonlServer {
    pub(super) fn handle_request(
        &mut self,
        request_id: u64,
        command: &str,
        arguments: &Map<String, Value>,
    ) -> Vec<Value> {
        match command {
            "initialize" => self.initialize(request_id, command, arguments),
            "launch" => self.launch(request_id, command, arguments),
            "attach" => vec![failure(
                request_id,
                command,
                "unsupported_capability",
                "Debugger attach is not supported.",
                "Use `launch`. The debugger owns an in-process VM and does not attach to a running process.",
            )],
            "step_back" | "reverse_continue" => vec![failure(
                request_id,
                command,
                "unsupported_capability",
                "Reverse execution is not supported.",
                "Use continue or a forward step. The debugger does not replay execution.",
            )],
            "replay" => vec![failure(
                request_id,
                command,
                "unsupported_capability",
                "Debugger replay is not supported.",
                "Forward execution can capture stops and queued input. Reverse playback is not available.",
            )],
            "reload" | "image.replace" => vec![failure(
                request_id,
                command,
                "unsupported_capability",
                "Live-image hot reload is not supported.",
                "Keep the current launch-owned executable. The debugger does not replace compiled code while stopped.",
            )],
            "record" => self.start_recording(request_id, command),
            "data_breakpoint.set" => vec![failure(
                request_id,
                command,
                "unsupported_capability",
                "Individual data-breakpoint set is not a protocol command.",
                "Use `data_breakpoints.replace` with identities from `location.describe`.",
            )],
            "data_breakpoints.replace" => {
                self.replace_data_breakpoints(request_id, command, arguments)
            }
            "breakpoint.set" => self.set_breakpoint(request_id, command, arguments),
            "breakpoint.clear" => self.clear_breakpoint(request_id, command, arguments),
            "function_breakpoints.replace" => {
                self.replace_function_breakpoints(request_id, command, arguments)
            }
            "runtime_failures.replace" => {
                self.replace_runtime_failure_filters(request_id, command, arguments)
            }
            "continue" => self.resume(request_id, command, ResumeCommand::Continue),
            "step_into" => {
                self.task_resume(request_id, command, arguments, ResumeCommand::StepInto)
            }
            "step_over" => {
                self.task_resume(request_id, command, arguments, ResumeCommand::StepOver)
            }
            "step_out" => self.task_resume(request_id, command, arguments, ResumeCommand::StepOut),
            "pause" => self.pause(request_id, command),
            "tasks" => self.tasks(request_id, command, arguments),
            "task.pause" => self.pause_task(request_id, command, arguments),
            "task.resume" => self.resume_task(request_id, command, arguments),
            "task.cancel" => self.cancel_task(request_id, command, arguments),
            "task.create" => self.create_task(request_id, command),
            "task.restart" => self.restart_task(request_id, command, arguments),
            "io.input" => self.push_debuggee_input(request_id, command, arguments),
            "io.eof" => self.signal_debuggee_eof(request_id, command),
            "io.cancel" => self.cancel_debuggee_input(request_id, command),
            "stack" => self.stack(request_id, command, arguments),
            "scopes" => self.scopes(request_id, command, arguments),
            "variables" => self.variables(request_id, command, arguments),
            "evaluate" => self.evaluate(request_id, command, arguments),
            "variable.set" => self.set_variable(request_id, command, arguments),
            "expression.set" => self.set_expression(request_id, command, arguments),
            "dictionary.insert" => self.insert_dictionary(request_id, command, arguments),
            "dictionary.remove" => self.remove_dictionary(request_id, command, arguments),
            "dictionary.replace_key" => self.replace_dictionary_key(request_id, command, arguments),
            "array.insert" => self.insert_array(request_id, command, arguments),
            "array.remove" => self.remove_array(request_id, command, arguments),
            "string.replace_character" => {
                self.replace_string_character(request_id, command, arguments)
            }
            "frame.return" => self.force_return(request_id, command, arguments),
            "frame.restart" => self.restart_frame(request_id, command, arguments),
            "instruction.set" => self.set_instruction(request_id, command, arguments),
            "location.describe" => self.describe_location(request_id, command, arguments),
            "recording.describe" => self.describe_recording(request_id, command),
            "task.result.replace" => {
                self.replace_completed_task_result(request_id, command, arguments)
            }
            "variant.describe" => self.describe_variant(request_id, command, arguments),
            "variant.construct" => self.construct_variant(request_id, command, arguments),
            "storage.initialize" => self.initialize_storage(request_id, command, arguments),
            "evaluate.cancel" => self.cancel_evaluation(request_id, command),
            "disconnect" => self.disconnect(request_id, command),
            _ => vec![failure(
                request_id,
                command,
                "unsupported_capability",
                format!("Debugger command `{command}` is not supported by protocol V2."),
                "Use a command advertised by `initialize`.",
            )],
        }
    }
}
