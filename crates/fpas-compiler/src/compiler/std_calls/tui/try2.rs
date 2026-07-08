use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower try-2 `Std.Tui` calls to VM intrinsics.
    pub(super) fn compile_tui_try2_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_TUI_DIALOG_NEW_MODAL => {
                self.expect_exact_args(s::STD_TUI_DIALOG_NEW_MODAL, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::DialogNewModal), location);
                Ok(true)
            }
            s::STD_TUI_BUTTON_NEW => {
                self.expect_exact_args(s::STD_TUI_BUTTON_NEW, 4, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::ButtonNew), location);
                Ok(true)
            }
            s::STD_TUI_DIALOG_ADD => {
                self.expect_exact_args(s::STD_TUI_DIALOG_ADD, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::DialogAdd), location);
                Ok(true)
            }
            s::STD_TUI_DIALOG_ADD_BUTTON => {
                self.expect_exact_args(s::STD_TUI_DIALOG_ADD_BUTTON, 5, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::DialogAddButton), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_EXEC_VIEW => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_EXEC_VIEW, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::ExecView), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_TRY2_INJECT_COMMAND => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_TRY2_INJECT_COMMAND,
                    2,
                    args,
                    location,
                )?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::Try2InjectCommand), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_TRY2_INJECT_KEYBOARD => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_TRY2_INJECT_KEYBOARD,
                    2,
                    args,
                    location,
                )?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::Try2InjectKeyboard),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_WINDOW_NEW => {
                self.expect_exact_args(s::STD_TUI_WINDOW_NEW, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::WindowNew), location);
                Ok(true)
            }
            s::STD_TUI_WINDOW_ADD => {
                self.expect_exact_args(s::STD_TUI_WINDOW_ADD, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::WindowAdd), location);
                Ok(true)
            }
            s::STD_TUI_DESKTOP_ADD => {
                self.expect_exact_args(s::STD_TUI_DESKTOP_ADD, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::DesktopAdd), location);
                Ok(true)
            }
            s::STD_TUI_STATIC_TEXT_NEW => {
                self.expect_exact_args(s::STD_TUI_STATIC_TEXT_NEW, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::StaticTextNew), location);
                Ok(true)
            }
            s::STD_TUI_CHECK_BOX_NEW => {
                self.expect_exact_args(s::STD_TUI_CHECK_BOX_NEW, 3, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::CheckBoxNew), location);
                Ok(true)
            }
            s::STD_TUI_INPUT_LINE_NEW => {
                self.expect_exact_args(s::STD_TUI_INPUT_LINE_NEW, 3, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::InputLineNew), location);
                Ok(true)
            }
            s::STD_TUI_CHECK_BOX_CHECKED => {
                self.expect_exact_args(s::STD_TUI_CHECK_BOX_CHECKED, 1, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::CheckBoxChecked), location);
                Ok(true)
            }
            s::STD_TUI_CHECK_BOX_SET_CHECKED => {
                self.expect_exact_args(s::STD_TUI_CHECK_BOX_SET_CHECKED, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::CheckBoxSetChecked),
                    location,
                );
                Ok(true)
            }
            s::STD_TUI_INPUT_LINE_TEXT => {
                self.expect_exact_args(s::STD_TUI_INPUT_LINE_TEXT, 1, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::InputLineText), location);
                Ok(true)
            }
            s::STD_TUI_INPUT_LINE_SET_TEXT => {
                self.expect_exact_args(s::STD_TUI_INPUT_LINE_SET_TEXT, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::InputLineSetText), location);
                Ok(true)
            }
            s::STD_TUI_MENU_BAR_NEW => {
                self.expect_exact_args(s::STD_TUI_MENU_BAR_NEW, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::MenuBarNew), location);
                Ok(true)
            }
            s::STD_TUI_STATUS_LINE_NEW => {
                self.expect_exact_args(s::STD_TUI_STATUS_LINE_NEW, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::StatusLineNew), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
