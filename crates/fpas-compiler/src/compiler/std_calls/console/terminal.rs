use crate::error::CompileError;
use fpas_bytecode::{ConsoleIntrinsic, Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Console` mode, device, and terminal-control calls.
    pub(super) fn compile_console_terminal_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_CONSOLE_DELAY => {
                self.expect_exact_args(s::STD_CONSOLE_DELAY, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::Delay), location);
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
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::EnableRawMode),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_DISABLE_RAW_MODE => {
                self.expect_zero_args(s::STD_CONSOLE_DISABLE_RAW_MODE, args, location)?;
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::DisableRawMode),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_ENTER_ALT_SCREEN => {
                self.expect_zero_args(s::STD_CONSOLE_ENTER_ALT_SCREEN, args, location)?;
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::EnterAltScreen),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_LEAVE_ALT_SCREEN => {
                self.expect_zero_args(s::STD_CONSOLE_LEAVE_ALT_SCREEN, args, location)?;
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::LeaveAltScreen),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_ENABLE_MOUSE => {
                self.expect_zero_args(s::STD_CONSOLE_ENABLE_MOUSE, args, location)?;
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::EnableMouse),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_DISABLE_MOUSE => {
                self.expect_zero_args(s::STD_CONSOLE_DISABLE_MOUSE, args, location)?;
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::DisableMouse),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_ENABLE_FOCUS => {
                self.expect_zero_args(s::STD_CONSOLE_ENABLE_FOCUS, args, location)?;
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::EnableFocus),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_DISABLE_FOCUS => {
                self.expect_zero_args(s::STD_CONSOLE_DISABLE_FOCUS, args, location)?;
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::DisableFocus),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_ENABLE_PASTE => {
                self.expect_zero_args(s::STD_CONSOLE_ENABLE_PASTE, args, location)?;
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::EnablePaste),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_DISABLE_PASTE => {
                self.expect_zero_args(s::STD_CONSOLE_DISABLE_PASTE, args, location)?;
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::DisablePaste),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_ACQUIRE_INTERACTIVE_TERMINAL => {
                self.expect_zero_args(s::STD_CONSOLE_ACQUIRE_INTERACTIVE_TERMINAL, args, location)?;
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::AcquireInteractiveTerminal),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_RELEASE_INTERACTIVE_TERMINAL => {
                self.expect_zero_args(s::STD_CONSOLE_RELEASE_INTERACTIVE_TERMINAL, args, location)?;
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::ReleaseInteractiveTerminal),
                    location,
                );
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
