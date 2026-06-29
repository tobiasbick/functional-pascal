use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Tui.Application*` lifecycle and event calls.
    pub(super) fn compile_tui_application_call(
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
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::ApplicationConfigure),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_RUN => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_RUN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::ApplicationRun), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_SIZE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_SIZE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::ApplicationSize), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_REQUEST_REDRAW => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_REQUEST_REDRAW, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_CREATE_DIALOG => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_CREATE_DIALOG, 3, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::CreateDialog), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_CREATE_BUTTON => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_CREATE_BUTTON, 4, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::CreateButton), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_ADD_CHILD => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_ADD_CHILD, 3, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::AddChild), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_ON_COMMAND => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_ON_COMMAND, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::RegisterOnCommand), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_PUMP => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_PUMP, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::Pump), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUIT => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_QUIT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::Quit), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_TEST_CLICK_BUTTON => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_TEST_CLICK_BUTTON,
                    2,
                    args,
                    location,
                )?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::TestClickButton), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
