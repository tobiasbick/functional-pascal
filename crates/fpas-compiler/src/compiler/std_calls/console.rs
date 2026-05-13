//! Lowers `Std.Console` calls to VM intrinsics and print operations.
//!
//! **Documentation:** `docs/pascal/std/console.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{ConsoleIntrinsic, Intrinsic, Op, SourceLocation, Value};
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
            s::STD_CONSOLE_WRITE_LN => {
                if args.is_empty() {
                    self.emit_constant(Value::Str(String::new()), location)?;
                    self.emit(Op::PrintLn, location);
                } else {
                    for (index, arg) in args.iter().enumerate() {
                        self.compile_expr(arg)?;
                        if index + 1 == args.len() {
                            self.emit(Op::PrintLn, location);
                        } else {
                            self.emit(Op::Print, location);
                        }
                    }
                }
                self.emit(Op::Unit, location);
                Ok(true)
            }
            s::STD_CONSOLE_WRITE => {
                for arg in args {
                    self.compile_expr(arg)?;
                    self.emit(Op::Print, location);
                }
                self.emit(Op::Unit, location);
                Ok(true)
            }
            s::STD_CONSOLE_READ_LN => {
                self.expect_zero_args(s::STD_CONSOLE_READ_LN, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::ReadLn), location);
                Ok(true)
            }
            s::STD_CONSOLE_READ => {
                self.expect_zero_args(s::STD_CONSOLE_READ, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::Read), location);
                Ok(true)
            }
            s::STD_CONSOLE_READ_KEY => {
                self.expect_zero_args(s::STD_CONSOLE_READ_KEY, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::ReadKey), location);
                Ok(true)
            }
            s::STD_CONSOLE_KEY_PRESSED => {
                self.expect_zero_args(s::STD_CONSOLE_KEY_PRESSED, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::KeyPressed), location);
                Ok(true)
            }
            s::STD_CONSOLE_READ_KEY_EVENT => {
                self.expect_zero_args(s::STD_CONSOLE_READ_KEY_EVENT, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::ReadKeyEvent), location);
                Ok(true)
            }
            s::STD_CONSOLE_EVENT_PENDING => {
                self.expect_zero_args(s::STD_CONSOLE_EVENT_PENDING, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::EventPending), location);
                Ok(true)
            }
            s::STD_CONSOLE_READ_EVENT => {
                self.expect_zero_args(s::STD_CONSOLE_READ_EVENT, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::ReadEvent), location);
                Ok(true)
            }
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
            s::STD_CONSOLE_TEXT_COLOR => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_COLOR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::TextColor), location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_BACKGROUND => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_BACKGROUND, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::TextBackground), location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_COLOR_RGB => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_COLOR_RGB, 3, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::TextColorRGB), location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_BACKGROUND_RGB => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_BACKGROUND_RGB, 3, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::TextBackgroundRGB), location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_COLOR_256 => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_COLOR_256, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::TextColor256), location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_BACKGROUND_256 => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_BACKGROUND_256, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::TextBackground256), location);
                Ok(true)
            }
            s::STD_CONSOLE_HIGH_VIDEO => {
                self.expect_zero_args(s::STD_CONSOLE_HIGH_VIDEO, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::HighVideo), location);
                Ok(true)
            }
            s::STD_CONSOLE_LOW_VIDEO => {
                self.expect_zero_args(s::STD_CONSOLE_LOW_VIDEO, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::LowVideo), location);
                Ok(true)
            }
            s::STD_CONSOLE_NORM_VIDEO => {
                self.expect_zero_args(s::STD_CONSOLE_NORM_VIDEO, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::NormVideo), location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_ATTR => {
                self.expect_zero_args(s::STD_CONSOLE_TEXT_ATTR, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::TextAttr), location);
                Ok(true)
            }
            s::STD_CONSOLE_SET_TEXT_ATTR => {
                self.expect_exact_args(s::STD_CONSOLE_SET_TEXT_ATTR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::SetTextAttr), location);
                Ok(true)
            }
            s::STD_CONSOLE_DELAY => {
                self.expect_exact_args(s::STD_CONSOLE_DELAY, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::Delay), location);
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
            s::STD_CONSOLE_TEXT_MODE => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_MODE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::TextMode), location);
                Ok(true)
            }
            s::STD_CONSOLE_LAST_MODE => {
                self.expect_zero_args(s::STD_CONSOLE_LAST_MODE, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::LastMode), location);
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
            s::STD_CONSOLE_SOUND => {
                self.expect_exact_args(s::STD_CONSOLE_SOUND, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::Sound), location);
                Ok(true)
            }
            s::STD_CONSOLE_NO_SOUND => {
                self.expect_zero_args(s::STD_CONSOLE_NO_SOUND, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::NoSound), location);
                Ok(true)
            }
            s::STD_CONSOLE_ASSIGN_CRT => {
                self.expect_zero_args(s::STD_CONSOLE_ASSIGN_CRT, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::AssignCrt), location);
                Ok(true)
            }
            s::STD_CONSOLE_ENABLE_RAW_MODE => {
                self.expect_zero_args(s::STD_CONSOLE_ENABLE_RAW_MODE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::EnableRawMode), location);
                Ok(true)
            }
            s::STD_CONSOLE_DISABLE_RAW_MODE => {
                self.expect_zero_args(s::STD_CONSOLE_DISABLE_RAW_MODE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::DisableRawMode), location);
                Ok(true)
            }
            s::STD_CONSOLE_ENTER_ALT_SCREEN => {
                self.expect_zero_args(s::STD_CONSOLE_ENTER_ALT_SCREEN, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::EnterAltScreen), location);
                Ok(true)
            }
            s::STD_CONSOLE_LEAVE_ALT_SCREEN => {
                self.expect_zero_args(s::STD_CONSOLE_LEAVE_ALT_SCREEN, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::LeaveAltScreen), location);
                Ok(true)
            }
            s::STD_CONSOLE_ENABLE_MOUSE => {
                self.expect_zero_args(s::STD_CONSOLE_ENABLE_MOUSE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::EnableMouse), location);
                Ok(true)
            }
            s::STD_CONSOLE_DISABLE_MOUSE => {
                self.expect_zero_args(s::STD_CONSOLE_DISABLE_MOUSE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::DisableMouse), location);
                Ok(true)
            }
            s::STD_CONSOLE_ENABLE_FOCUS => {
                self.expect_zero_args(s::STD_CONSOLE_ENABLE_FOCUS, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::EnableFocus), location);
                Ok(true)
            }
            s::STD_CONSOLE_DISABLE_FOCUS => {
                self.expect_zero_args(s::STD_CONSOLE_DISABLE_FOCUS, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::DisableFocus), location);
                Ok(true)
            }
            s::STD_CONSOLE_ENABLE_PASTE => {
                self.expect_zero_args(s::STD_CONSOLE_ENABLE_PASTE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::EnablePaste), location);
                Ok(true)
            }
            s::STD_CONSOLE_DISABLE_PASTE => {
                self.expect_zero_args(s::STD_CONSOLE_DISABLE_PASTE, args, location)?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::DisablePaste), location);
                Ok(true)
            }
            s::STD_CONSOLE_READ_EVENT_TIMEOUT => {
                self.expect_exact_args(s::STD_CONSOLE_READ_EVENT_TIMEOUT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::ReadEventTimeout), location);
                Ok(true)
            }
            s::STD_CONSOLE_POLL_EVENT => {
                self.expect_zero_args(s::STD_CONSOLE_POLL_EVENT, args, location)?;
                self.emit_intrinsic(Intrinsic::Console(ConsoleIntrinsic::PollEvent), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
