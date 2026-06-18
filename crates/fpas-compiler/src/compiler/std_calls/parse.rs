//! Lowers `Std.Parse` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/text/parse.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, ParseIntrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_parse_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_PARSE_TRY_INT => {
                self.expect_exact_args(name, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Parse(ParseIntrinsic::TryInt), location);
                Ok(true)
            }
            s::STD_PARSE_TRY_REAL => {
                self.expect_exact_args(name, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Parse(ParseIntrinsic::TryReal), location);
                Ok(true)
            }
            s::STD_PARSE_TRY_BOOL => {
                self.expect_exact_args(name, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Parse(ParseIntrinsic::TryBool), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
