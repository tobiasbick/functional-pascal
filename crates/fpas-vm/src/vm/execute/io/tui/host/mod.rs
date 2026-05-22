//! `Std.Tui` host-control intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui-app.md` (from the repository root).

mod lifecycle;
mod process;
mod redraw;

use crate::vm::Worker;
use crate::vm::diagnostics::VmError;
use crate::vm::runtime_error;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_diagnostics::codes::RUNTIME_INTRINSIC_STACK_STATE_ERROR;

impl Worker {
    /// Executes host-control `Std.Tui` intrinsics.
    pub(super) fn try_exec_tui_host_intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        line: SourceLocation,
    ) -> Result<bool, VmError> {
        match intrinsic {
            Intrinsic::Tui(TuiIntrinsic::HostRegisterOnKeyPressed) => {
                self.register_tui_handler(
                    2,
                    "OnKeyPressed",
                    "Pass a `function (Application, Std.Console.KeyEvent): boolean`.",
                    |tui, function| tui.on_key_pressed = Some(function),
                    line,
                )?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostRegisterOnResize) => {
                self.register_tui_handler(
                    2,
                    "OnResize",
                    "Pass a `procedure (Application, Std.Tui.Size)` (two parameters).",
                    |tui, function| tui.on_resize = Some(function),
                    line,
                )?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostProcessNext) => {
                let max_spins = self.pop_int(line)?.max(0).min(4096) as usize;
                self.pop_tui_application(line)?;
                let tag = self.tui_host_process_next_inner(max_spins, line)?;
                self.push(fpas_bytecode::Value::Integer(tag))?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaint) => {
                self.register_tui_handler(
                    1,
                    "OnPaint",
                    "Pass a `procedure (Application)` (one parameter).",
                    |tui, function| tui.on_paint = Some(function),
                    line,
                )?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostRegisterOnIdle) => {
                let function = self.pop(line)?;
                let milliseconds = self.pop_int(line)?.max(0);
                self.pop_tui_application(line)?;
                self.validate_host_handler_function(
                    &function,
                    1,
                    "OnIdle",
                    "Pass `Application`, an idle interval in milliseconds, and a `procedure (Application)` handler.",
                    line,
                )?;
                self.with_tui(|tui| {
                    tui.on_idle = Some(function);
                    tui.idle_interval_ms = milliseconds;
                });
            }
            Intrinsic::Tui(TuiIntrinsic::HostDispatchRedraw) => {
                self.pop_tui_application(line)?;
                let tag = self.tui_host_dispatch_redraw_inner(line)?;
                self.push(fpas_bytecode::Value::Integer(tag))?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostRunLoop) => {
                let max_iters = self.pop_int(line)?.max(0).min(1_000_000) as usize;
                self.pop_tui_application(line)?;
                self.tui_host_run_loop_inner(max_iters, line)?;
                self.push(fpas_bytecode::Value::Unit)?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostRequestQuit) => {
                self.pop_tui_application(line)?;
                self.with_tui(|tui| tui.quit_requested = true);
            }
            Intrinsic::Tui(TuiIntrinsic::HostRegisterOnExit) => {
                self.register_tui_handler(
                    2,
                    "OnExit",
                    "Pass a `procedure (Application, Std.Tui.ExitReason)` (two parameters).",
                    |tui, function| tui.on_exit = Some(function),
                    line,
                )?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostRegisterOnMouse) => {
                self.register_tui_handler(
                    2,
                    "OnMouse",
                    "Pass a `procedure (Application, Std.Console.Event)` (two parameters).",
                    |tui, function| tui.on_mouse = Some(function),
                    line,
                )?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaste) => {
                self.register_tui_handler(
                    2,
                    "OnPaste",
                    "Pass a `procedure (Application, Std.Console.Event)` (two parameters).",
                    |tui, function| tui.on_paste = Some(function),
                    line,
                )?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusGained) => {
                self.register_tui_handler(
                    2,
                    "OnFocusGained",
                    "Pass a `procedure (Application, Std.Console.Event)` (two parameters).",
                    |tui, function| tui.on_focus_gained = Some(function),
                    line,
                )?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusLost) => {
                self.register_tui_handler(
                    2,
                    "OnFocusLost",
                    "Pass a `procedure (Application, Std.Console.Event)` (two parameters).",
                    |tui, function| tui.on_focus_lost = Some(function),
                    line,
                )?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostRegisterOnActivate) => {
                self.register_tui_handler(
                    1,
                    "OnActivate",
                    "Pass a `procedure (Application)` (one parameter).",
                    |tui, function| tui.on_activate = Some(function),
                    line,
                )?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostRegisterOnDeactivate) => {
                self.register_tui_handler(
                    1,
                    "OnDeactivate",
                    "Pass a `procedure (Application)` (one parameter).",
                    |tui, function| tui.on_deactivate = Some(function),
                    line,
                )?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostRegisterOnCommand) => {
                self.register_tui_handler(
                    2,
                    "OnCommand",
                    "Pass a `procedure (Application, integer)` (two parameters).",
                    |tui, function| tui.on_command = Some(function),
                    line,
                )?;
            }
            Intrinsic::Tui(TuiIntrinsic::HostInvokeOnKeyPressed) => {
                let key_event = self.pop_console_key_event(line)?;
                self.pop_tui_application(line)?;
                let handler = self.with_tui(|tui| tui.on_key_pressed.clone());
                let handler = handler.ok_or_else(|| {
                    runtime_error(
                        RUNTIME_INTRINSIC_STACK_STATE_ERROR,
                        "No OnKeyPressed handler is registered for the Tui host",
                        "Call `TuiHostRegisterOnKeyPressed` after `Application.Open` with a `function (Application, Std.Console.KeyEvent): boolean`.",
                        line,
                    )
                })?;
                let app_rec = Self::tui_application_record();
                let consumed = self.call_function_sync(
                    &handler,
                    &[app_rec, Self::key_event_record(key_event)],
                    line,
                )?;
                self.push(consumed)?;
            }
            _ => return Ok(false),
        }

        Ok(true)
    }
}
