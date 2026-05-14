use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Tui.Application.Host*` lifecycle and repaint callback calls.
    pub(super) fn compile_tui_host_lifecycle_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_RESIZE => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_RESIZE,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnResize),
                    location,
                );
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
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnPaint),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_HOST_REGISTER_ON_IDLE => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_HOST_REGISTER_ON_IDLE,
                    3,
                    args,
                    location,
                )?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnIdle),
                    location,
                );
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
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnExit),
                    location,
                );
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
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnActivate),
                    location,
                );
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
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::HostRegisterOnDeactivate),
                    location,
                );
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
