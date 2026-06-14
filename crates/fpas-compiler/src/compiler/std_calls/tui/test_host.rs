use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower headless native TUI testing calls.
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
            s::STD_TUI_APPLICATION_TEST_PUMP => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_TEST_PUMP, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::TestPump), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_TEST_PUMP_UNTIL_IDLE => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_TEST_PUMP_UNTIL_IDLE,
                    1,
                    args,
                    location,
                )?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::TestPumpUntilIdle), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_CLOSE_FOR_TEST => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_CLOSE_FOR_TEST, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::CloseForTest), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
