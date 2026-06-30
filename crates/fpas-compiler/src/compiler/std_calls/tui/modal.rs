use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Tui` modal and dialog calls.
    pub(super) fn compile_tui_modal_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_TUI_APPLICATION_SHOW_MODAL => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_SHOW_MODAL, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::ApplicationShowModal),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_SHOW_DIALOG => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_SHOW_DIALOG, 6, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(
                    Intrinsic::Tui(TuiIntrinsic::ApplicationShowDialog),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_CLOSE_MODAL => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_CLOSE_MODAL, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::ApplicationCloseModal),
                    location,
                );
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
