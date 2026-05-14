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
                self.emit_intrinsic(
                    Intrinsic::Tui(TuiIntrinsic::ApplicationReadEventTimeout),
                    location,
                );
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
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::ApplicationRequestRedraw),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_REDRAW_PENDING => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_REDRAW_PENDING, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(
                    Intrinsic::Tui(TuiIntrinsic::ApplicationRedrawPending),
                    location,
                );
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
