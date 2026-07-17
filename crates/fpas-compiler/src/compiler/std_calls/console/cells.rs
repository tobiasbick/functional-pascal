//! Lowering for cell-oriented `Std.Console` operations.

use crate::error::CompileError;
use fpas_bytecode::{ConsoleIntrinsic, Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::super::super::Compiler;

impl Compiler {
    /// Lower `Std.Console` color, cell, rectangle, and display-width calls.
    pub(super) fn compile_console_cell_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        let (arity, intrinsic, returns_value) = match name {
            s::STD_CONSOLE_CRT_COLOR => (1, ConsoleIntrinsic::CrtColor, true),
            s::STD_CONSOLE_ANSI_256_COLOR => (1, ConsoleIntrinsic::Ansi256Color, true),
            s::STD_CONSOLE_RGB_COLOR => (3, ConsoleIntrinsic::RgbColor, true),
            s::STD_CONSOLE_PUT_CELL => (3, ConsoleIntrinsic::PutCell, false),
            s::STD_CONSOLE_GET_CELL => (2, ConsoleIntrinsic::GetCell, true),
            s::STD_CONSOLE_FILL_RECT => (2, ConsoleIntrinsic::FillRect, false),
            s::STD_CONSOLE_WRITE_CELLS => (3, ConsoleIntrinsic::WriteCells, false),
            s::STD_CONSOLE_DISPLAY_WIDTH => (1, ConsoleIntrinsic::DisplayWidth, true),
            s::STD_CONSOLE_GRAPHEME_WIDTH => (1, ConsoleIntrinsic::GraphemeWidth, true),
            _ => return Ok(false),
        };

        self.expect_exact_args(name, arity, args, location)?;
        for arg in args {
            self.compile_expr(arg)?;
        }
        let intrinsic = Intrinsic::Console(intrinsic);
        if returns_value {
            self.emit_intrinsic(intrinsic, location);
        } else {
            self.emit_intrinsic_unit(intrinsic, location);
        }
        Ok(true)
    }
}
