//! Lowers `Std.Console` calls to VM intrinsics and print operations.
//!
//! **Documentation:** `docs/pascal/std/console/README.md` (from the repository root).

mod io;
mod screen;
mod style;
mod terminal;

use crate::error::CompileError;
use fpas_bytecode::SourceLocation;
use fpas_parser::Expr;

use super::super::Compiler;

impl Compiler {
    /// Lower a `Std.Console` call to the corresponding print operation or intrinsic.
    pub(super) fn compile_console_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        if self.compile_console_io_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_console_screen_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_console_style_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_console_terminal_call(name, args, location)? {
            return Ok(true);
        }
        Ok(false)
    }
}
