//! Lowers `Std.Path` calls to runtime intrinsics.
//!
//! **Documentation:** `docs/pascal/std/path.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, PathIntrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    /// Compile a `Std.Path` call into a runtime intrinsic when `name` belongs to the unit.
    pub(super) fn compile_path_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_PATH_JOIN => {
                self.expect_exact_args(s::STD_PATH_JOIN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Path(PathIntrinsic::Join), location);
                Ok(true)
            }
            s::STD_PATH_BASE_NAME => {
                self.expect_exact_args(s::STD_PATH_BASE_NAME, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Path(PathIntrinsic::BaseName), location);
                Ok(true)
            }
            s::STD_PATH_DIR_NAME => {
                self.expect_exact_args(s::STD_PATH_DIR_NAME, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Path(PathIntrinsic::DirName), location);
                Ok(true)
            }
            s::STD_PATH_EXTENSION => {
                self.expect_exact_args(s::STD_PATH_EXTENSION, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Path(PathIntrinsic::Extension), location);
                Ok(true)
            }
            s::STD_PATH_NORMALIZE => {
                self.expect_exact_args(s::STD_PATH_NORMALIZE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Path(PathIntrinsic::Normalize), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
