//! Keyboard command resolution and `OnKeyPressed` fallback.

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH;
use fpas_std::{CommandEvent, ConsoleKeyEvent, ProcessOutcome};

impl Worker {
    /// Resolves global command bindings, then dispatches the `OnKeyPressed` fallback.
    pub(super) fn dispatch_tui_key_event(
        &mut self,
        key_event: ConsoleKeyEvent,
        on_key: Option<Value>,
        app_rec: Value,
        line: SourceLocation,
    ) -> Result<ProcessOutcome, VmError> {
        if let Some(command_id) = self.with_tui(|tui| tui.commands.resolve(&key_event)) {
            return self.dispatch_tui_command(CommandEvent::resolve(command_id, None), line);
        }

        if let Some(handler) = on_key {
            let consumed = self.call_function_sync_allowing_shutdown(
                &handler,
                &[app_rec, Self::key_event_record(key_event)],
                line,
            )?;
            match consumed {
                Value::Boolean(consumed) => Ok(ProcessOutcome::Key {
                    handled: true,
                    consumed,
                }),
                other => Err(runtime_error(
                    RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                    format!(
                        "OnKeyPressed must return boolean, got {}",
                        other.type_name()
                    ),
                    "Return `true` when the application consumed the key or `false` otherwise.",
                    line,
                )),
            }
        } else {
            Ok(ProcessOutcome::Key {
                handled: false,
                consumed: false,
            })
        }
    }
}
