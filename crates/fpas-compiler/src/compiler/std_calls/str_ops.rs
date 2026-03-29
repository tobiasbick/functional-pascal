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
            s::STD_STR_REPEAT => {
                self.expect_exact_args(s::STD_STR_REPEAT, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::StrRepeat, location);
                Ok(true)
            }
            s::STD_STR_PAD_LEFT => {
                self.expect_exact_args(s::STD_STR_PAD_LEFT, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::StrPadLeft, location);
                Ok(true)
            }
            s::STD_STR_PAD_RIGHT => {
                self.expect_exact_args(s::STD_STR_PAD_RIGHT, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::StrPadRight, location);
                Ok(true)
            }
            s::STD_STR_PAD_CENTER => {
                self.expect_exact_args(s::STD_STR_PAD_CENTER, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::StrPadCenter, location);
                Ok(true)
            }
            s::STD_STR_FROM_CHAR => {
                self.expect_exact_args(s::STD_STR_FROM_CHAR, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::StrFromChar, location);
                Ok(true)
            }
            s::STD_STR_CHAR_AT => {
                self.expect_exact_args(s::STD_STR_CHAR_AT, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::StrCharAt, location);
                Ok(true)
            }
            s::STD_STR_SET_CHAR_AT => {
                self.expect_exact_args(s::STD_STR_SET_CHAR_AT, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::StrSetCharAt, location);
                Ok(true)
            }
            s::STD_STR_ORD => {
                self.expect_exact_args(s::STD_STR_ORD, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::StrOrd, location);
                Ok(true)
            }
            s::STD_STR_CHR => {
                self.expect_exact_args(s::STD_STR_CHR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::StrChr, location);
                Ok(true)
            }
            s::STD_STR_INSERT => {
                self.expect_exact_args(s::STD_STR_INSERT, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::StrInsert, location);
                Ok(true)
            }
            s::STD_STR_DELETE => {
                self.expect_exact_args(s::STD_STR_DELETE, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::StrDelete, location);
                Ok(true)
            }
            s::STD_STR_REVERSE => {
                self.expect_exact_args(s::STD_STR_REVERSE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::StrReverse, location);
                Ok(true)
            }
            s::STD_STR_TRIM_LEFT => {
                self.expect_exact_args(s::STD_STR_TRIM_LEFT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::StrTrimLeft, location);
                Ok(true)
            }
            s::STD_STR_TRIM_RIGHT => {
                self.expect_exact_args(s::STD_STR_TRIM_RIGHT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::StrTrimRight, location);
                Ok(true)
            }
            s::STD_STR_LAST_INDEX_OF => {
                self.expect_exact_args(s::STD_STR_LAST_INDEX_OF, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::StrLastIndexOf, location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
