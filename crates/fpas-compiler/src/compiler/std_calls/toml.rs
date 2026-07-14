//! Lowers `Std.Toml` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/text/toml.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TomlIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_toml_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_TOML_PARSE => {
                self.expect_exact_args(s::STD_TOML_PARSE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Toml(TomlIntrinsic::Parse), location);
                Ok(true)
            }
            s::STD_TOML_STRINGIFY => {
                self.expect_exact_args(s::STD_TOML_STRINGIFY, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Toml(TomlIntrinsic::Stringify), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
