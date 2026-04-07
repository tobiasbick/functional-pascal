//! Lowers `Std.Console` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/console.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_console_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_CONSOLE_READ_LN => {
                self.expect_zero_args(s::STD_CONSOLE_READ_LN, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleReadLn, location);
                Ok(true)
            }
            s::STD_CONSOLE_READ => {
                self.expect_zero_args(s::STD_CONSOLE_READ, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleRead, location);
                Ok(true)
            }
            s::STD_CONSOLE_READ_KEY => {
                self.expect_zero_args(s::STD_CONSOLE_READ_KEY, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleReadKey, location);
                Ok(true)
            }
            s::STD_CONSOLE_KEY_PRESSED => {
                self.expect_zero_args(s::STD_CONSOLE_KEY_PRESSED, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleKeyPressed, location);
                Ok(true)
            }
            s::STD_CONSOLE_READ_KEY_EVENT => {
                self.expect_zero_args(s::STD_CONSOLE_READ_KEY_EVENT, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleReadKeyEvent, location);
                Ok(true)
            }
            s::STD_CONSOLE_EVENT_PENDING => {
                self.expect_zero_args(s::STD_CONSOLE_EVENT_PENDING, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleEventPending, location);
                Ok(true)
            }
            s::STD_CONSOLE_READ_EVENT => {
                self.expect_zero_args(s::STD_CONSOLE_READ_EVENT, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleReadEvent, location);
                Ok(true)
            }
            s::STD_CONSOLE_CLR_SCR => {
                self.expect_zero_args(s::STD_CONSOLE_CLR_SCR, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleClrScr, location);
                Ok(true)
            }
            s::STD_CONSOLE_CLR_EOL => {
                self.expect_zero_args(s::STD_CONSOLE_CLR_EOL, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleClrEol, location);
                Ok(true)
            }
            s::STD_CONSOLE_GOTO_XY => {
                self.expect_exact_args(s::STD_CONSOLE_GOTO_XY, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleGotoXY, location);
                Ok(true)
            }
            s::STD_CONSOLE_WHERE_X => {
                self.expect_zero_args(s::STD_CONSOLE_WHERE_X, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleWhereX, location);
                Ok(true)
            }
            s::STD_CONSOLE_WHERE_Y => {
                self.expect_zero_args(s::STD_CONSOLE_WHERE_Y, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleWhereY, location);
                Ok(true)
            }
            s::STD_CONSOLE_WIND_MIN => {
                self.expect_zero_args(s::STD_CONSOLE_WIND_MIN, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleWindMin, location);
                Ok(true)
            }
            s::STD_CONSOLE_WIND_MAX => {
                self.expect_zero_args(s::STD_CONSOLE_WIND_MAX, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleWindMax, location);
                Ok(true)
            }
            s::STD_CONSOLE_DEL_LINE => {
                self.expect_zero_args(s::STD_CONSOLE_DEL_LINE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleDelLine, location);
                Ok(true)
            }
            s::STD_CONSOLE_INS_LINE => {
                self.expect_zero_args(s::STD_CONSOLE_INS_LINE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleInsLine, location);
                Ok(true)
            }
            s::STD_CONSOLE_WINDOW => {
                self.expect_exact_args(s::STD_CONSOLE_WINDOW, 4, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::ConsoleWindow, location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_COLOR => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_COLOR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleTextColor, location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_BACKGROUND => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_BACKGROUND, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleTextBackground, location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_COLOR_RGB => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_COLOR_RGB, 3, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::ConsoleTextColorRGB, location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_BACKGROUND_RGB => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_BACKGROUND_RGB, 3, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::ConsoleTextBackgroundRGB, location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_COLOR_256 => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_COLOR_256, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleTextColor256, location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_BACKGROUND_256 => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_BACKGROUND_256, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleTextBackground256, location);
                Ok(true)
            }
            s::STD_CONSOLE_HIGH_VIDEO => {
                self.expect_zero_args(s::STD_CONSOLE_HIGH_VIDEO, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleHighVideo, location);
                Ok(true)
            }
            s::STD_CONSOLE_LOW_VIDEO => {
                self.expect_zero_args(s::STD_CONSOLE_LOW_VIDEO, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleLowVideo, location);
                Ok(true)
            }
            s::STD_CONSOLE_NORM_VIDEO => {
                self.expect_zero_args(s::STD_CONSOLE_NORM_VIDEO, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleNormVideo, location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_ATTR => {
                self.expect_zero_args(s::STD_CONSOLE_TEXT_ATTR, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleTextAttr, location);
                Ok(true)
            }
            s::STD_CONSOLE_SET_TEXT_ATTR => {
                self.expect_exact_args(s::STD_CONSOLE_SET_TEXT_ATTR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleSetTextAttr, location);
                Ok(true)
            }
            s::STD_CONSOLE_DELAY => {
                self.expect_exact_args(s::STD_CONSOLE_DELAY, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleDelay, location);
                Ok(true)
            }
            s::STD_CONSOLE_CURSOR_ON => {
                self.expect_zero_args(s::STD_CONSOLE_CURSOR_ON, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleCursorOn, location);
                Ok(true)
            }
            s::STD_CONSOLE_CURSOR_OFF => {
                self.expect_zero_args(s::STD_CONSOLE_CURSOR_OFF, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleCursorOff, location);
                Ok(true)
            }
            s::STD_CONSOLE_CURSOR_BIG => {
                self.expect_zero_args(s::STD_CONSOLE_CURSOR_BIG, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleCursorBig, location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_MODE => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_MODE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleTextMode, location);
                Ok(true)
            }
            s::STD_CONSOLE_LAST_MODE => {
                self.expect_zero_args(s::STD_CONSOLE_LAST_MODE, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleLastMode, location);
                Ok(true)
            }
            s::STD_CONSOLE_SCREEN_WIDTH => {
                self.expect_zero_args(s::STD_CONSOLE_SCREEN_WIDTH, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleScreenWidth, location);
                Ok(true)
            }
            s::STD_CONSOLE_SCREEN_HEIGHT => {
                self.expect_zero_args(s::STD_CONSOLE_SCREEN_HEIGHT, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsoleScreenHeight, location);
                Ok(true)
            }
            s::STD_CONSOLE_SOUND => {
                self.expect_exact_args(s::STD_CONSOLE_SOUND, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleSound, location);
                Ok(true)
            }
            s::STD_CONSOLE_NO_SOUND => {
                self.expect_zero_args(s::STD_CONSOLE_NO_SOUND, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleNoSound, location);
                Ok(true)
            }
            s::STD_CONSOLE_ASSIGN_CRT => {
                self.expect_zero_args(s::STD_CONSOLE_ASSIGN_CRT, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleAssignCrt, location);
                Ok(true)
            }
            s::STD_CONSOLE_ENABLE_RAW_MODE => {
                self.expect_zero_args(s::STD_CONSOLE_ENABLE_RAW_MODE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleEnableRawMode, location);
                Ok(true)
            }
            s::STD_CONSOLE_DISABLE_RAW_MODE => {
                self.expect_zero_args(s::STD_CONSOLE_DISABLE_RAW_MODE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleDisableRawMode, location);
                Ok(true)
            }
            s::STD_CONSOLE_ENTER_ALT_SCREEN => {
                self.expect_zero_args(s::STD_CONSOLE_ENTER_ALT_SCREEN, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleEnterAltScreen, location);
                Ok(true)
            }
            s::STD_CONSOLE_LEAVE_ALT_SCREEN => {
                self.expect_zero_args(s::STD_CONSOLE_LEAVE_ALT_SCREEN, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleLeaveAltScreen, location);
                Ok(true)
            }
            s::STD_CONSOLE_ENABLE_MOUSE => {
                self.expect_zero_args(s::STD_CONSOLE_ENABLE_MOUSE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleEnableMouse, location);
                Ok(true)
            }
            s::STD_CONSOLE_DISABLE_MOUSE => {
                self.expect_zero_args(s::STD_CONSOLE_DISABLE_MOUSE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleDisableMouse, location);
                Ok(true)
            }
            s::STD_CONSOLE_ENABLE_FOCUS => {
                self.expect_zero_args(s::STD_CONSOLE_ENABLE_FOCUS, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleEnableFocus, location);
                Ok(true)
            }
            s::STD_CONSOLE_DISABLE_FOCUS => {
                self.expect_zero_args(s::STD_CONSOLE_DISABLE_FOCUS, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleDisableFocus, location);
                Ok(true)
            }
            s::STD_CONSOLE_ENABLE_PASTE => {
                self.expect_zero_args(s::STD_CONSOLE_ENABLE_PASTE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleEnablePaste, location);
                Ok(true)
            }
            s::STD_CONSOLE_DISABLE_PASTE => {
                self.expect_zero_args(s::STD_CONSOLE_DISABLE_PASTE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::ConsoleDisablePaste, location);
                Ok(true)
            }
            s::STD_CONSOLE_READ_EVENT_TIMEOUT => {
                self.expect_exact_args(s::STD_CONSOLE_READ_EVENT_TIMEOUT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ConsoleReadEventTimeout, location);
                Ok(true)
            }
            s::STD_CONSOLE_POLL_EVENT => {
                self.expect_zero_args(s::STD_CONSOLE_POLL_EVENT, args, location)?;
                self.emit_intrinsic(Intrinsic::ConsolePollEvent, location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
