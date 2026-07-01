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
            s::STD_TUI_APPLICATION_TEST_SEND_KEY => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_TEST_SEND_KEY, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::TestSendKey), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_TEST_SEND_MOUSE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_TEST_SEND_MOUSE, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::TestSendMouse), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_TEST_MOVE_MOUSE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_TEST_MOVE_MOUSE, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::TestMoveMouse), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_TEST_CLICK_MOUSE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_TEST_CLICK_MOUSE, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::TestClickMouse), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_TEST_RESIZE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_TEST_RESIZE, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::TestResize), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_TEST_PASTE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_TEST_PASTE, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::TestPaste), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_TEST_FOCUS => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_TEST_FOCUS, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::TestFocus), location);
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
            _ => Ok(false),
        }
    }
}
