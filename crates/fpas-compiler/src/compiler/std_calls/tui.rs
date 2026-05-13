//! Lowers `Std.Tui` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui.md`, `docs/pascal/std/tui-app.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
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
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_CLOSE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_CLOSE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::ApplicationClose), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_CONFIGURE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_CONFIGURE, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::ApplicationConfigure), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_RUN => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_RUN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::ApplicationRun), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_SHOW_MODAL => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_SHOW_MODAL, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::ApplicationShowModal), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_SHOW_DIALOG => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_SHOW_DIALOG, 6, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.compile_expr(&args[3])?;
                self.compile_expr(&args[4])?;
                self.compile_expr(&args[5])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::ApplicationShowDialog), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_CLOSE_MODAL => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_CLOSE_MODAL, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::ApplicationCloseModal), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_SIZE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_SIZE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::ApplicationSize), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_READ_EVENT => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_READ_EVENT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::ApplicationReadEvent), location);
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
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::ApplicationReadEventTimeout), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_POLL_EVENT => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_POLL_EVENT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::ApplicationPollEvent), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_REQUEST_REDRAW => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_REQUEST_REDRAW, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_REDRAW_PENDING => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_REDRAW_PENDING, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::ApplicationRedrawPending), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_POLL_NEXT => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_HOST_POLL_NEXT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::HostPollNext), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnKeyPressed), location);
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
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::HostInvokeOnKeyPressed), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnResize), location);
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
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::HostProcessNext), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaint), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnIdle), location);
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
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::HostDispatchRedraw), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_RUN_LOOP => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_HOST_RUN_LOOP, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRunLoop), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRequestQuit), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnExit), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnMouse), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaste), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusGained), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnFocusLost), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnActivate), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnDeactivate), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnCommand), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostBindCommand), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_BIND_COMMAND_TO_VIEW => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_BIND_COMMAND_TO_VIEW,
                    4,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.compile_expr(&args[3])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostBindCommandToView), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_BIND_COMMAND_TO_ACTIVE_MODAL => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_BIND_COMMAND_TO_ACTIVE_MODAL,
                    3,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostBindCommandToActiveModal), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_ENTER_MODAL => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_HOST_ENTER_MODAL, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostEnterModal), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_LEAVE_MODAL => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_HOST_LEAVE_MODAL, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostLeaveModal), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_MODAL_DEPTH => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_HOST_MODAL_DEPTH, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::HostModalDepth), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_VIEW => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_VIEW,
                    5,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.compile_expr(&args[3])?;
                self.compile_expr(&args[4])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::HostRegisterView), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostUnregisterView), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostPushChildView), location);
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
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::HostQueryFocusedViewId), location);
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
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostAttachViewToActiveModal), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_SET_VIEW_RECT => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_SET_VIEW_RECT,
                    6,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.compile_expr(&args[3])?;
                self.compile_expr(&args[4])?;
                self.compile_expr(&args[5])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostSetViewRect), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_SET_VIEW_PARENT => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_SET_VIEW_PARENT,
                    3,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostSetViewParent), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_VIEW_PAINT => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_VIEW_PAINT,
                    3,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::HostRegisterOnViewPaint), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
