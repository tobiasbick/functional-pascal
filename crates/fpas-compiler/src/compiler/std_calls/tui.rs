//! Lowers `Std.Tui` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_tui_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_TUI_APPLICATION_OPEN => {
                self.expect_zero_args(s::STD_TUI_APPLICATION_OPEN, args, location)?;
                self.emit_intrinsic(Intrinsic::TuiApplicationOpen, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_CLOSE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_CLOSE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::TuiApplicationClose, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_CONFIGURE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_CONFIGURE, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiApplicationConfigure, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_RUN => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_RUN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::TuiApplicationRun, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_SIZE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_SIZE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::TuiApplicationSize, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_READ_EVENT => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_READ_EVENT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::TuiApplicationReadEvent, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_READ_EVENT_TIMEOUT => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_READ_EVENT_TIMEOUT,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::TuiApplicationReadEventTimeout, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_POLL_EVENT => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_POLL_EVENT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::TuiApplicationPollEvent, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_REQUEST_REDRAW => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_REQUEST_REDRAW, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::TuiApplicationRequestRedraw, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_REDRAW_PENDING => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_REDRAW_PENDING, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::TuiApplicationRedrawPending, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_POLL_NEXT => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_HOST_POLL_NEXT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::TuiHostPollNext, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_KEY_PRESSED => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_KEY_PRESSED,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostRegisterOnKeyPressed, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_INVOKE_ON_KEY_PRESSED => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_INVOKE_ON_KEY_PRESSED,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::TuiHostInvokeOnKeyPressed, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_RESIZE => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_RESIZE,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostRegisterOnResize, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_PROCESS_NEXT => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_PROCESS_NEXT,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::TuiHostProcessNext, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_PAINT => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_PAINT,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostRegisterOnPaint, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_IDLE => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_IDLE,
                    3,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostRegisterOnIdle, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_DISPATCH_REDRAW => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_DISPATCH_REDRAW,
                    1,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::TuiHostDispatchRedraw, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_RUN_LOOP => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_HOST_RUN_LOOP, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostRunLoop, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REQUEST_QUIT => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REQUEST_QUIT,
                    1,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostRequestQuit, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_EXIT => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_EXIT,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostRegisterOnExit, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_MOUSE => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_MOUSE,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostRegisterOnMouse, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_PASTE => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_PASTE,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostRegisterOnPaste, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_GAINED => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_GAINED,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostRegisterOnFocusGained, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_LOST => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_FOCUS_LOST,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostRegisterOnFocusLost, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_ACTIVATE => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_ACTIVATE,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostRegisterOnActivate, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_DEACTIVATE => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_DEACTIVATE,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostRegisterOnDeactivate, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_COMMAND => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_COMMAND,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostRegisterOnCommand, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_BIND_COMMAND => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_BIND_COMMAND,
                    3,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostBindCommand, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_ENTER_MODAL => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_HOST_ENTER_MODAL, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostEnterModal, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_LEAVE_MODAL => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_HOST_LEAVE_MODAL, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostLeaveModal, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_MODAL_DEPTH => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_HOST_MODAL_DEPTH, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::TuiHostModalDepth, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_VIEW => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_HOST_REGISTER_VIEW, 5, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.compile_expr(&args[3])?;
                self.compile_expr(&args[4])?;
                self.emit_intrinsic(Intrinsic::TuiHostRegisterView, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_UNREGISTER_VIEW => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_UNREGISTER_VIEW,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostUnregisterView, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_PUSH_CHILD_VIEW => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_PUSH_CHILD_VIEW,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostPushChildView, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_QUERY_FOCUSED_VIEW_ID => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_QUERY_FOCUSED_VIEW_ID,
                    1,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::TuiHostQueryFocusedViewId, location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_ATTACH_VIEW_TO_ACTIVE_MODAL => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_ATTACH_VIEW_TO_ACTIVE_MODAL,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::TuiHostAttachViewToActiveModal, location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
