//! Lowers `Std.Tui` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui.md` (from the repository root).

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
            _ => Ok(false),
        }
    }
}
