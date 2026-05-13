//! Lowers `Std.Dict` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/dict.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{DictIntrinsic, Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_dict_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_DICT_LENGTH => {
                self.expect_exact_args(s::STD_DICT_LENGTH, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Dict(DictIntrinsic::Length), location);
                Ok(true)
            }
            s::STD_DICT_CONTAINS_KEY => {
                self.expect_exact_args(s::STD_DICT_CONTAINS_KEY, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Dict(DictIntrinsic::ContainsKey), location);
                Ok(true)
            }
            s::STD_DICT_KEYS => {
                self.expect_exact_args(s::STD_DICT_KEYS, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Dict(DictIntrinsic::Keys), location);
                Ok(true)
            }
            s::STD_DICT_VALUES => {
                self.expect_exact_args(s::STD_DICT_VALUES, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Dict(DictIntrinsic::Values), location);
                Ok(true)
            }
            s::STD_DICT_REMOVE => {
                self.expect_exact_args(s::STD_DICT_REMOVE, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Dict(DictIntrinsic::Remove), location);
                Ok(true)
            }
            s::STD_DICT_GET => {
                self.expect_exact_args(s::STD_DICT_GET, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Dict(DictIntrinsic::Get), location);
                Ok(true)
            }
            s::STD_DICT_MERGE => {
                self.expect_exact_args(s::STD_DICT_MERGE, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Dict(DictIntrinsic::Merge), location);
                Ok(true)
            }
            s::STD_DICT_MAP => {
                self.expect_exact_args(s::STD_DICT_MAP, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Dict(DictIntrinsic::Map), location);
                Ok(true)
            }
            s::STD_DICT_FILTER => {
                self.expect_exact_args(s::STD_DICT_FILTER, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Dict(DictIntrinsic::Filter), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
