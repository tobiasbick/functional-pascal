use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TuiIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Tui.Application` lifecycle, chrome, and test-helper calls.
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
            s::STD_TUI_APPLICATION_NEW => {
                self.expect_zero_args(s::STD_TUI_APPLICATION_NEW, args, location)?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::ApplicationOpen), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_CLOSE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_CLOSE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::ApplicationClose), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_RUN => {
                if args.len() == 1 {
                    self.expect_exact_args(s::STD_TUI_APPLICATION_RUN, 1, args, location)?;
                    self.compile_expr(&args[0])?;
                    self.emit_intrinsic_unit(
                        Intrinsic::Tui(TuiIntrinsic::ApplicationRun),
                        location,
                    );
                } else if args.len() == 2 {
                    self.expect_exact_args(s::STD_TUI_APPLICATION_RUN, 2, args, location)?;
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                    self.emit_intrinsic_unit(
                        Intrinsic::Tui(TuiIntrinsic::ApplicationRunWithOnCommand),
                        location,
                    );
                } else {
                    self.expect_exact_args(s::STD_TUI_APPLICATION_RUN, 1, args, location)?;
                }
                Ok(true)
            }
            s::STD_TUI_APPLICATION_SIZE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_SIZE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Tui(TuiIntrinsic::ApplicationSize), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_SET_MENU_BAR => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_SET_MENU_BAR, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::SetMenuBar), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_SET_STATUS_LINE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_SET_STATUS_LINE, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::SetStatusLine), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_QUIT => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_QUIT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::Quit), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_ON_KEY => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_ON_KEY, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::RegisterOnKey), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_ON_MOUSE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_ON_MOUSE, 2, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::RegisterOnMouse), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_TEST_CLICK_BUTTON | s::STD_TUI_TEST_CLICK => {
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
            s::STD_TUI_APPLICATION_TEST_CLICK_MOUSE => {
                self.expect_exact_args(s::STD_TUI_APPLICATION_TEST_CLICK_MOUSE, 3, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Tui(TuiIntrinsic::TestClickMouse), location);
                Ok(true)
            }
            s::STD_TUI_APPLICATION_TEST_DISPATCH_MENU_COMMAND | s::STD_TUI_TEST_DISPATCH_MENU => {
                self.expect_exact_args(
                    s::STD_TUI_APPLICATION_TEST_DISPATCH_MENU_COMMAND,
                    4,
                    args,
                    location,
                )?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Tui(TuiIntrinsic::TestDispatchMenuCommand),
                    location,
                );
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
