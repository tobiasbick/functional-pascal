//! Turbo Vision bridge command callback dispatch (upstream `CM_*` ids without offset translation).
//!
//! **Documentation:** `docs/pascal/std/tui/app/vm-bridge.md`

use crate::vm::Worker;
use crate::vm::diagnostics::{VmError, runtime_error};
use fpas_bytecode::{SourceLocation, Value};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;
use turbo_vision::core::command::CommandId;

/// Maps an upstream command id to the integer passed to FPAS `OnCommand`.
#[must_use]
pub(in crate::vm::execute::io::tui::bridge) fn bridge_command_for_callback(
    command: CommandId,
) -> i64 {
    i64::from(command)
}

/// Invokes the registered `OnCommand` handler with the upstream command id unchanged.
pub(in crate::vm::execute::io::tui::bridge) fn bridge_dispatch_on_command(
    worker: &mut Worker,
    command: CommandId,
    line: SourceLocation,
) -> Result<(), VmError> {
    let handler = worker.with_tui(|tui| tui.on_command.clone());
    let Some(handler) = handler else {
        return Err(runtime_error(
            RUNTIME_INTRINSIC_STACK_STATE_ERROR,
            "Application.Run requires an `Application.OnCommand` handler on the Turbo Vision path",
            "Call `Application.OnCommand(App, OnCommand)` before `Application.Run(App)`.",
            line,
        ));
    };
    bridge_dispatch_on_command_handler(worker, &handler, command, line)
}

/// Invokes a specific `OnCommand` handler (used when the handler is passed at run time later).
pub(in crate::vm::execute::io::tui::bridge) fn bridge_dispatch_on_command_handler(
    worker: &mut Worker,
    handler: &Value,
    command: CommandId,
    line: SourceLocation,
) -> Result<(), VmError> {
    worker.validate_host_handler_function(
        handler,
        2,
        "OnCommand",
        "Pass a `procedure (Application, integer)` command handler.",
        line,
    )?;
    let app_rec = Worker::tui_application_record();
    let _ = worker.call_function_sync_allowing_shutdown(
        handler,
        &[
            app_rec,
            Value::Integer(bridge_command_for_callback(command)),
        ],
        line,
    )?;
    Ok(())
}

#[cfg(test)]
impl Worker {
    /// Dispatches a command id through the registered Turbo Vision `OnCommand` handler.
    pub(crate) fn bridge_dispatch_command_event_for_tests(
        &mut self,
        command: CommandId,
        line: SourceLocation,
    ) -> Result<(), VmError> {
        bridge_dispatch_on_command(self, command, line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use turbo_vision::core::command::{CM_OK, CM_QUIT};

    #[test]
    fn command_ids_pass_through_unchanged() {
        assert_eq!(bridge_command_for_callback(CM_OK), 10);
        assert_eq!(bridge_command_for_callback(CM_QUIT), 1);
        assert_eq!(bridge_command_for_callback(24), 24);
        assert_eq!(bridge_command_for_callback(4096), 4096);
    }
}
