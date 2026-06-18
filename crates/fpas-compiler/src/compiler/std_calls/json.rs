//! Lowers `Std.Json` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/text/json.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, JsonIntrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_json_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_JSON_PARSE => {
                self.expect_exact_args(s::STD_JSON_PARSE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Json(JsonIntrinsic::Parse), location);
                Ok(true)
            }
            s::STD_JSON_STRINGIFY => {
                self.expect_exact_args(s::STD_JSON_STRINGIFY, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Json(JsonIntrinsic::Stringify), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
