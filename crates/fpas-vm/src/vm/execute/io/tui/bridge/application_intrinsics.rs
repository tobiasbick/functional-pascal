//! Turbo Vision bridge `Std.Tui` application lifecycle intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};

impl Worker {
    /// Executes application-level `Std.Tui` intrinsics.
    pub(in crate::vm::execute::io::tui) fn try_exec_tui_application_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Tui(TuiIntrinsic::ApplicationOpen) => {
                self.reset_tui_session_state();
                {
                    let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console(|console| tui.session.open_deferred(console, line))?;
                }
                self.open_bridge_session();
                self.push(Self::tui_application_record())?;
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationClose) => {
                self.pop_tui_application(line)?;
                self.close_tui_application_state(line)?;
            }
            // The compiler lowers `Application.Run` with `emit_intrinsic_unit`, which emits the
            // statement-level `Op::Unit` itself; pushing another unit here would leak one stack
            // slot and shift every local declared after `Run`.
            Intrinsic::Tui(TuiIntrinsic::ApplicationRun) => {
                super::bridge_application_run(self, line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationRunWithOnCommand) => {
                let handler = self.pop(line)?;
                self.pop_tui_application(line)?;
                self.with_tui(|tui| tui.on_command = Some(handler));
                super::bridge_application_run_loop(self, line)?;
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationSize) => {
                self.pop_tui_application(line)?;
                let (width, height) = {
                    let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console(|console| tui.session.size(console, line))?
                };
                self.push(Self::tui_size_record(width, height))?;
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationConfigure) => {
                let handlers = self.pop_tui_application_handlers(line)?;
                self.pop_tui_application(line)?;

                let on_command = Self::required_record_field(&handlers, "OnCommand", line)?.clone();
                self.validate_host_handler_function(
                    &on_command,
                    2,
                    "OnCommand",
                    "Set `OnCommand := Handler` where `Handler` is `procedure (Application, integer)`.",
                    line,
                )?;
                let on_key = self.optional_host_handler_field(
                    &handlers,
                    "OnKey",
                    2,
                    "OnKey",
                    "Set `OnKey := Some(Handler)` or `None`; the handler must be `function (Application, Std.Console.KeyEvent): boolean`.",
                    line,
                )?;
                let on_mouse = self.optional_host_handler_field(
                    &handlers,
                    "OnMouse",
                    2,
                    "OnMouse",
                    "Set `OnMouse := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Console.Event)`.",
                    line,
                )?;

                self.with_tui(|tui| {
                    tui.on_command = Some(on_command);
                    tui.turbo_vision_on_key = on_key;
                    tui.turbo_vision_on_mouse = on_mouse;
                });
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
