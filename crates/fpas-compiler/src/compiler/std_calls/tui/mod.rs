//! Lowers `Std.Tui` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/tui/session.md`, `docs/pascal/std/tui/app/README.md` (from the repository root).

mod application;
mod bridge;
mod test_host;

use crate::error::CompileError;
use fpas_bytecode::SourceLocation;
use fpas_parser::Expr;

use super::super::Compiler;

impl Compiler {
    /// Lower a `Std.Tui` call to the corresponding VM intrinsic.
    pub(super) fn compile_tui_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        if self.compile_tui_application_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_tui_bridge_call(name, args, location)? {
            return Ok(true);
        }
        if self.compile_tui_test_host_call(name, args, location)? {
            return Ok(true);
        }
        Ok(false)
    }
}
