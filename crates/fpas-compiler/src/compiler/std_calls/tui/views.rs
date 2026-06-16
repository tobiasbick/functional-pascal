use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Tui.Application.Host*` view-management calls.
    pub(super) fn compile_tui_view_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_TUI_APPLICATION_HOST_BIND_COMMAND_TO_VIEW => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_BIND_COMMAND_TO_VIEW,
                    4,
                    args,
                    location,
                )?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostBindCommandToView),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_VIEW => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_VIEW,
                    5,
                    args,
                    location,
                )?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
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
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostUnregisterView),
                    location,
                );
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
            s::STD_TUI_APPLICATION_HOST_SET_VIEW_RECT => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_SET_VIEW_RECT,
                    6,
                    args,
                    location,
                )?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
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
                for arg in args {
                    self.compile_expr(arg)?;
                }
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
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnViewPaint),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_CREATE_SOLID_FILL_VIEW => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_CREATE_SOLID_FILL_VIEW,
                    8,
                    args,
                    location,
                )?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(
                    Intrinsic::Tui(TuiIntrinsic::HostCreateSolidFillView),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_CREATE_MENU_BAR_VIEW => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_CREATE_MENU_BAR_VIEW,
                    7,
                    args,
                    location,
                )?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(
                    Intrinsic::Tui(TuiIntrinsic::HostCreateMenuBarView),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_SET_MENU_BAR_ITEMS => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_SET_MENU_BAR_ITEMS,
                    3,
                    args,
                    location,
                )?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostSetMenuBarItems),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_CREATE_STATUS_BAR_VIEW => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_CREATE_STATUS_BAR_VIEW,
                    7,
                    args,
                    location,
                )?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(
                    Intrinsic::Tui(TuiIntrinsic::HostCreateStatusBarView),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_SET_STATUS_BAR_SEGMENTS => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_SET_STATUS_BAR_SEGMENTS,
                    3,
                    args,
                    location,
                )?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostSetStatusBarSegments),
                    location,
                );
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
