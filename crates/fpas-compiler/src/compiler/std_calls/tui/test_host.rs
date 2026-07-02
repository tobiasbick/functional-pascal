use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower headless Turbo Vision testing calls.
    pub(super) fn compile_tui_test_host_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_TUI_APPLICATION_OPEN_FOR_TEST => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_OPEN_FOR_TEST, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::OpenForTest), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_CLOSE_FOR_TEST => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_CLOSE_FOR_TEST, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::CloseForTest), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_TEST_SET_FILE_DIALOG_RESULT => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_TEST_SET_FILE_DIALOG_RESULT,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::TestSetFileDialogResult),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_APPLICATION_TEST_SET_DIALOG_RESULT => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_TEST_SET_DIALOG_RESULT,
                    2,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::TestSetDialogResult),
                    location,
                );
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
