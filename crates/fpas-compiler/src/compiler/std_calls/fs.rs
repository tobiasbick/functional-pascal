//! Lowers `Std.Fs` calls to runtime intrinsics.
//!
//! **Documentation:** `docs/pascal/std/host/fs.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{FsIntrinsic, Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    /// Compile a `Std.Fs` call into a runtime intrinsic when `name` belongs to the unit.
    pub(super) fn compile_fs_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_FS_READ_TEXT => {
                self.expect_exact_args(s::STD_FS_READ_TEXT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Fs(FsIntrinsic::ReadText), location);
                Ok(true)
            }
            s::STD_FS_WRITE_TEXT => {
                self.expect_exact_args(s::STD_FS_WRITE_TEXT, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Fs(FsIntrinsic::WriteText), location);
                Ok(true)
            }
            s::STD_FS_EXISTS => {
                self.expect_exact_args(s::STD_FS_EXISTS, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Fs(FsIntrinsic::Exists), location);
                Ok(true)
            }
            s::STD_FS_IS_FILE => {
                self.expect_exact_args(s::STD_FS_IS_FILE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Fs(FsIntrinsic::IsFile), location);
                Ok(true)
            }
            s::STD_FS_IS_DIR => {
                self.expect_exact_args(s::STD_FS_IS_DIR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Fs(FsIntrinsic::IsDir), location);
                Ok(true)
            }
            s::STD_FS_CREATE_DIR => {
                self.expect_exact_args(s::STD_FS_CREATE_DIR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Fs(FsIntrinsic::CreateDir), location);
                Ok(true)
            }
            s::STD_FS_GLOB => {
                self.expect_exact_args(s::STD_FS_GLOB, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Fs(FsIntrinsic::Glob), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
