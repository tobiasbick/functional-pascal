use crate::error::CompileError;
use fpas_bytecode::{ConsoleIntrinsic, Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Console` color and text-style calls.
    pub(super) fn compile_console_style_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_CONSOLE_TEXT_COLOR => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_COLOR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Console(ConsoleIntrinsic::TextColor), location);
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_BACKGROUND => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_BACKGROUND, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::TextBackground),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_COLOR_RGB => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_COLOR_RGB, 3, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::TextColorRGB),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_BACKGROUND_RGB => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_BACKGROUND_RGB, 3, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::TextBackgroundRGB),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_COLOR_256 => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_COLOR_256, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::TextColor256),
                    location,
                );
                Ok(true)
            }
            s::STD_CONSOLE_TEXT_BACKGROUND_256 => {
                self.expect_exact_args(s::STD_CONSOLE_TEXT_BACKGROUND_256, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::TextBackground256),
                    location,
                );
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
                self.emit_intrinsic_unit(
                    Intrinsic::Console(ConsoleIntrinsic::SetTextAttr),
                    location,
                );
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
