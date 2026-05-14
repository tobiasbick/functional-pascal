use crate::error::CompileError;
use fpas_bytecode::{ConsoleIntrinsic, Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Console` cursor, window, and screen-state calls.
    pub(super) fn compile_console_screen_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_CONSOLE_CLR_SCR => {
                self.expect_zero_args(s::STD_CONSOLE_CLR_SCR, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::ClrScr), location);
                Ok(true)
            }
            s::STD_CONSOLE_CLR_EOL => {
                self.expect_zero_args(s::STD_CONSOLE_CLR_EOL, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::ClrEol), location);
                Ok(true)
            }
            s::STD_CONSOLE_GOTO_XY => {
                self.expect_exact_args(s::STD_CONSOLE_GOTO_XY, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::GotoXY), location);
                Ok(true)
            }
            s::STD_CONSOLE_WHERE_X => {
                self.expect_zero_args(s::STD_CONSOLE_WHERE_X, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::WhereX), location);
                Ok(true)
            }
            s::STD_CONSOLE_WHERE_Y => {
                self.expect_zero_args(s::STD_CONSOLE_WHERE_Y, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::WhereY), location);
                Ok(true)
            }
            s::STD_CONSOLE_WIND_MIN => {
                self.expect_zero_args(s::STD_CONSOLE_WIND_MIN, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::WindMin), location);
                Ok(true)
            }
            s::STD_CONSOLE_WIND_MAX => {
                self.expect_zero_args(s::STD_CONSOLE_WIND_MAX, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::WindMax), location);
                Ok(true)
            }
            s::STD_CONSOLE_DEL_LINE => {
                self.expect_zero_args(s::STD_CONSOLE_DEL_LINE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::DelLine), location);
                Ok(true)
            }
            s::STD_CONSOLE_INS_LINE => {
                self.expect_zero_args(s::STD_CONSOLE_INS_LINE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::InsLine), location);
                Ok(true)
            }
            s::STD_CONSOLE_WINDOW => {
                self.expect_exact_args(s::STD_CONSOLE_WINDOW, 4, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::Window), location);
                Ok(true)
            }
            s::STD_CONSOLE_CURSOR_ON => {
                self.expect_zero_args(s::STD_CONSOLE_CURSOR_ON, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::CursorOn), location);
                Ok(true)
            }
            s::STD_CONSOLE_CURSOR_OFF => {
                self.expect_zero_args(s::STD_CONSOLE_CURSOR_OFF, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::CursorOff), location);
                Ok(true)
            }
            s::STD_CONSOLE_CURSOR_BIG => {
                self.expect_zero_args(s::STD_CONSOLE_CURSOR_BIG, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::CursorBig), location);
                Ok(true)
            }
            s::STD_CONSOLE_SCREEN_WIDTH => {
                self.expect_zero_args(s::STD_CONSOLE_SCREEN_WIDTH, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::ScreenWidth), location);
                Ok(true)
            }
            s::STD_CONSOLE_SCREEN_HEIGHT => {
                self.expect_zero_args(s::STD_CONSOLE_SCREEN_HEIGHT, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::ScreenHeight), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
