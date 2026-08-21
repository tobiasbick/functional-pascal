//! Typed command routing for the debug engine.

use super::DebugEngine;
use super::actor::ResumeCommand;
use super::error::EngineFailure;
use super::record::DebugRecord;
use super::reply::{fail, unsupported};
use super::request::DebugOp;

impl DebugEngine {
    pub(super) fn handle_request(&mut self, request_id: u64, op: DebugOp) -> Vec<DebugRecord> {
        let command = op.command().name().to_string();
        let command = command.as_str();
        match op {
            DebugOp::Initialize => self.initialize(request_id, command),
            DebugOp::Launch { stop_on_entry } => self.launch(request_id, command, stop_on_entry),
            DebugOp::Attach => vec![unsupported(
                request_id,
                command,
                "Debugger attach is not supported.",
                "Use `launch`. The debugger owns an in-process VM and does not attach to a running process.",
            )],
            DebugOp::StepBack | DebugOp::ReverseContinue => vec![unsupported(
                request_id,
                command,
                "Reverse execution is not supported.",
                "Use continue or a forward step. The debugger does not replay execution.",
            )],
            DebugOp::Replay => vec![unsupported(
                request_id,
                command,
                "Debugger replay is not supported.",
                "Forward execution can capture stops and queued input. Reverse playback is not available.",
            )],
            DebugOp::Reload | DebugOp::ImageReplace => self.reload_live_image(request_id, command),
            DebugOp::ImageRollback => self.rollback_live_image(request_id, command),
            DebugOp::ReloadClassify => self.classify_live_image(request_id, command),
            DebugOp::Record => self.start_recording(request_id, command),
            DebugOp::DataBreakpointSet => vec![unsupported(
                request_id,
                command,
                "Individual data-breakpoint set is not a protocol command.",
                "Use `data_breakpoints.replace` with identities from `location.describe`.",
            )],
            DebugOp::DataBreakpointsReplace { breakpoints } => {
                self.replace_data_breakpoints(request_id, command, breakpoints)
            }
            DebugOp::BreakpointSet {
                source,
                line,
                column,
                assign,
                condition,
                hit_condition,
                log_message,
            } => self.set_breakpoint(
                request_id,
                command,
                source,
                line,
                column,
                assign,
                condition,
                hit_condition,
                log_message,
            ),
            DebugOp::BreakpointClear { breakpoint_id } => {
                self.clear_breakpoint(request_id, command, breakpoint_id)
            }
            DebugOp::FunctionBreakpointsReplace { breakpoints } => {
                self.replace_function_breakpoints(request_id, command, breakpoints)
            }
            DebugOp::RuntimeFailuresReplace { filters } => {
                self.replace_runtime_failure_filters(request_id, command, filters)
            }
            DebugOp::Continue => self.resume(request_id, command, ResumeCommand::Continue),
            DebugOp::StepInto { task_id } => {
                self.resume(request_id, command, ResumeCommand::StepInto(task_id))
            }
            DebugOp::StepOver { task_id } => {
                self.resume(request_id, command, ResumeCommand::StepOver(task_id))
            }
            DebugOp::StepOut { task_id } => {
                self.resume(request_id, command, ResumeCommand::StepOut(task_id))
            }
            DebugOp::Pause => self.pause(request_id, command),
            DebugOp::Tasks { start, count } => self.tasks(request_id, command, start, count),
            DebugOp::TaskPause { task_id } => self.pause_task(request_id, command, task_id),
            DebugOp::TaskResume { task_id } => self.resume_task(request_id, command, task_id),
            DebugOp::TaskCancel { task_id } => self.cancel_task(request_id, command, task_id),
            DebugOp::TaskCreate => self.create_task(request_id, command),
            DebugOp::TaskRestart { task_id } => self.restart_task(request_id, command, task_id),
            DebugOp::IoInput { text } => self.push_debuggee_input(request_id, command, text),
            DebugOp::IoEof => self.signal_debuggee_eof(request_id, command),
            DebugOp::IoCancel => self.cancel_debuggee_input(request_id, command),
            DebugOp::Stack {
                start,
                count,
                task_id,
            } => self.stack(request_id, command, start, count, task_id),
            DebugOp::Scopes { frame_id } => self.scopes(request_id, command, frame_id),
            DebugOp::Variables {
                variables_reference,
                start,
                count,
            } => self.variables(request_id, command, variables_reference, start, count),
            DebugOp::Evaluate {
                expression,
                frame_id,
                async_eval,
            } => self.evaluate(request_id, command, expression, frame_id, async_eval),
            DebugOp::VariableSet {
                variables_reference,
                name,
                expression,
            } => self.set_variable(request_id, command, variables_reference, name, expression),
            DebugOp::ExpressionSet {
                target,
                expression,
                frame_id,
            } => self.set_expression(request_id, command, target, expression, frame_id),
            DebugOp::DictionaryInsert {
                target,
                key,
                expression,
                frame_id,
            } => self.insert_dictionary(request_id, command, target, key, expression, frame_id),
            DebugOp::DictionaryRemove {
                target,
                key,
                frame_id,
            } => self.remove_dictionary(request_id, command, target, key, frame_id),
            DebugOp::DictionaryReplaceKey {
                target,
                key,
                new_key,
                frame_id,
            } => self.replace_dictionary_key(request_id, command, target, key, new_key, frame_id),
            DebugOp::ArrayInsert {
                target,
                index,
                expression,
                frame_id,
            } => self.insert_array(request_id, command, target, index, expression, frame_id),
            DebugOp::ArrayRemove {
                target,
                index,
                frame_id,
            } => self.remove_array(request_id, command, target, index, frame_id),
            DebugOp::StringReplaceCharacter {
                target,
                index,
                expression,
                frame_id,
            } => self
                .replace_string_character(request_id, command, target, index, expression, frame_id),
            DebugOp::FrameReturn {
                frame_id,
                expression,
            } => self.force_return(request_id, command, frame_id, expression),
            DebugOp::FrameRestart { frame_id } => self.restart_frame(request_id, command, frame_id),
            DebugOp::InstructionSet {
                frame_id,
                instruction,
            } => self.set_instruction(request_id, command, frame_id, instruction),
            DebugOp::LocationDescribe {
                variables_reference,
                name,
            } => self.describe_location(request_id, command, variables_reference, name),
            DebugOp::RecordingDescribe => self.describe_recording(request_id, command),
            DebugOp::TaskResultReplace {
                task_id,
                expression,
                frame_id,
            } => self
                .replace_completed_task_result(request_id, command, task_id, expression, frame_id),
            DebugOp::VariantDescribe { target, frame_id } => {
                self.describe_variant(request_id, command, target, frame_id)
            }
            DebugOp::VariantConstruct {
                target,
                variant,
                fields,
                frame_id,
            } => self.construct_variant(request_id, command, target, variant, fields, frame_id),
            DebugOp::StorageInitialize {
                target,
                initializer,
                expression,
                frame_id,
            } => self.initialize_storage(
                request_id,
                command,
                target,
                initializer,
                expression,
                frame_id,
            ),
            DebugOp::EvaluateCancel => self.cancel_evaluation(request_id, command),
            DebugOp::Disconnect => self.disconnect(request_id, command),
            DebugOp::Unknown(name) => vec![fail(
                request_id,
                &name,
                EngineFailure::unsupported_capability(&name),
            )],
        }
    }
}
