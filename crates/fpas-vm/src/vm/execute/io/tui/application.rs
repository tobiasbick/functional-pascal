//! `Std.Tui` application lifecycle and configuration intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic, Value};

impl Worker {
    /// Executes application-level `Std.Tui` intrinsics.
    pub(super) fn try_exec_tui_application_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Tui(TuiIntrinsic::ApplicationOpen) => {
                self.reset_tui_host_state();
                {
                    let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console(|console| tui.session.open_deferred(console, line))?;
                }
                self.push(Self::tui_application_record())?;
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationClose) => {
                self.pop_tui_application(line)?;
                if !self.request_tui_host_stop_for_active_run() {
                    self.close_tui_application_state(line)?;
                }
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationConfigure) => {
                let handlers = self.pop_tui_application_handlers(line)?;
                self.pop_tui_application(line)?;

                let on_paint = self.optional_host_handler_field(
                    &handlers,
                    "OnPaint",
                    1,
                    "OnPaint",
                    "Set `OnPaint := Some(Handler)` or `None`; the handler must be `procedure (Application)`.",
                    line,
                )?;
                let on_key_pressed = self.optional_host_handler_field(
                    &handlers,
                    "OnKeyPressed",
                    2,
                    "OnKeyPressed",
                    "Set `OnKeyPressed := Some(Handler)` or `None`; the handler must be `function (Application, Std.Console.KeyEvent): boolean`.",
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
                let on_paste = self.optional_host_handler_field(
                    &handlers,
                    "OnPaste",
                    2,
                    "OnPaste",
                    "Set `OnPaste := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Console.Event)`.",
                    line,
                )?;
                let on_focus_gained = self.optional_host_handler_field(
                    &handlers,
                    "OnFocusGained",
                    2,
                    "OnFocusGained",
                    "Set `OnFocusGained := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Console.Event)`.",
                    line,
                )?;
                let on_focus_lost = self.optional_host_handler_field(
                    &handlers,
                    "OnFocusLost",
                    2,
                    "OnFocusLost",
                    "Set `OnFocusLost := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Console.Event)`.",
                    line,
                )?;
                let on_activate = self.optional_host_handler_field(
                    &handlers,
                    "OnActivate",
                    1,
                    "OnActivate",
                    "Set `OnActivate := Some(Handler)` or `None`; the handler must be `procedure (Application)`.",
                    line,
                )?;
                let on_deactivate = self.optional_host_handler_field(
                    &handlers,
                    "OnDeactivate",
                    1,
                    "OnDeactivate",
                    "Set `OnDeactivate := Some(Handler)` or `None`; the handler must be `procedure (Application)`.",
                    line,
                )?;
                let on_command = self.optional_host_handler_field(
                    &handlers,
                    "OnCommand",
                    2,
                    "OnCommand",
                    "Set `OnCommand := Some(Handler)` or `None`; the handler must be `procedure (Application, integer)`.",
                    line,
                )?;
                let on_resize = self.optional_host_handler_field(
                    &handlers,
                    "OnResize",
                    2,
                    "OnResize",
                    "Set `OnResize := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Tui.Size)`.",
                    line,
                )?;
                let idle_interval_ms = self
                    .integer_record_field(&handlers, "OnIdleMilliseconds", line)?
                    .max(0);
                let on_idle = self.optional_host_handler_field(
                    &handlers,
                    "OnIdle",
                    1,
                    "OnIdle",
                    "Set `OnIdle := Some(Handler)` or `None`; the handler must be `procedure (Application)`.",
                    line,
                )?;
                let on_exit = self.optional_host_handler_field(
                    &handlers,
                    "OnExit",
                    2,
                    "OnExit",
                    "Set `OnExit := Some(Handler)` or `None`; the handler must be `procedure (Application, Std.Tui.ExitReason)`.",
                    line,
                )?;

                let mut tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                tui.on_paint = on_paint;
                tui.on_key_pressed = on_key_pressed;
                tui.on_mouse = on_mouse;
                tui.on_paste = on_paste;
                tui.on_focus_gained = on_focus_gained;
                tui.on_focus_lost = on_focus_lost;
                tui.on_activate = on_activate;
                tui.on_deactivate = on_deactivate;
                tui.on_command = on_command;
                tui.on_resize = on_resize;
                tui.idle_interval_ms = idle_interval_ms;
                tui.on_idle = on_idle;
                tui.on_exit = on_exit;
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationRun) => {
                self.tui_application_run(line)?;
                self.push(Value::Unit)?;
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationSize) => {
                self.pop_tui_application(line)?;
                let (width, height) = {
                    let tui = self.shared.tui.lock().unwrap_or_else(|e| e.into_inner());
                    self.with_console(|console| tui.session.size(console, line))?
                };
                self.push(Self::tui_size_record(width, height))?;
            }
            Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw) => {
                self.pop_tui_application(line)?;
                self.with_tui(|tui| tui.session.request_redraw(line))?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
