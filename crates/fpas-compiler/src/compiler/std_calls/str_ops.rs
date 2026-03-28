use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_str_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_STR_LENGTH => {
                self.expect_exact_args(s::STD_STR_LENGTH, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::StrLength, location);
                Ok(true)
            }
            s::STD_STR_TO_UPPER => {
                self.expect_exact_args(s::STD_STR_TO_UPPER, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::StrToUpper, location);
                Ok(true)
            }
            s::STD_STR_TO_LOWER => {
                self.expect_exact_args(s::STD_STR_TO_LOWER, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::StrToLower, location);
                Ok(true)
            }
            s::STD_STR_TRIM => {
                self.expect_exact_args(s::STD_STR_TRIM, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::StrTrim, location);
                Ok(true)
            }
            s::STD_STR_CONTAINS => {
                self.expect_exact_args(s::STD_STR_CONTAINS, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::StrContains, location);
                Ok(true)
            }
            s::STD_STR_STARTS_WITH => {
                self.expect_exact_args(s::STD_STR_STARTS_WITH, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::StrStartsWith, location);
                Ok(true)
            }
            s::STD_STR_ENDS_WITH => {
                self.expect_exact_args(s::STD_STR_ENDS_WITH, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::StrEndsWith, location);
                Ok(true)
            }
            s::STD_STR_SUBSTRING => {
                self.expect_exact_args(s::STD_STR_SUBSTRING, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::StrSubstring, location);
                Ok(true)
            }
            s::STD_STR_INDEX_OF => {
                self.expect_exact_args(s::STD_STR_INDEX_OF, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::StrIndexOf, location);
                Ok(true)
            }
            s::STD_STR_REPLACE => {
                self.expect_exact_args(s::STD_STR_REPLACE, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::StrReplace, location);
                Ok(true)
            }
            s::STD_STR_SPLIT => {
                self.expect_exact_args(s::STD_STR_SPLIT, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::StrSplit, location);
                Ok(true)
            }
            s::STD_STR_JOIN => {
                self.expect_exact_args(s::STD_STR_JOIN, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::StrJoin, location);
                Ok(true)
            }
            s::STD_STR_IS_NUMERIC => {
                self.expect_exact_args(s::STD_STR_IS_NUMERIC, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::StrIsNumeric, location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
