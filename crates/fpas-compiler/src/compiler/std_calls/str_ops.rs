//! Lowers `Std.Str` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/text/str.md` (from the repository root).

use crate::error::{CompileError, compile_error};
use fpas_bytecode::{Intrinsic, Op, SourceLocation, StrIntrinsic, Value};
use fpas_diagnostics::codes::COMPILE_INTRINSIC_ARITY_MISMATCH;
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
            s::STD_STR_FORMAT => {
                if args.is_empty() {
                    return Err(compile_error(
                        COMPILE_INTRINSIC_ARITY_MISMATCH,
                        "Format requires at least one argument (the template string)",
                        "Use: Format('template %d', Value)",
                        Self::call_site_span(location),
                    ));
                }
                // Stack layout consumed by StrFormat: template, arg1..argN, N
                self.compile_expr(&args[0])?;
                for arg in &args[1..] {
                    self.compile_expr(arg)?;
                }
                let arg_count = (args.len() - 1) as i64;
                self.emit_constant(Value::Integer(arg_count), location)?;
                self.emit(
                    Op::Intrinsic(u16::from(Intrinsic::Str(StrIntrinsic::Format))),
                    location,
                );
                Ok(true)
            }
            s::STD_STR_LENGTH => {
                self.expect_exact_args(s::STD_STR_LENGTH, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::Length), location);
                Ok(true)
            }
            s::STD_STR_TO_UPPER => {
                self.expect_exact_args(s::STD_STR_TO_UPPER, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::ToUpper), location);
                Ok(true)
            }
            s::STD_STR_TO_LOWER => {
                self.expect_exact_args(s::STD_STR_TO_LOWER, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::ToLower), location);
                Ok(true)
            }
            s::STD_STR_TRIM => {
                self.expect_exact_args(s::STD_STR_TRIM, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::Trim), location);
                Ok(true)
            }
            s::STD_STR_CONTAINS => {
                self.expect_exact_args(s::STD_STR_CONTAINS, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::Contains), location);
                Ok(true)
            }
            s::STD_STR_STARTS_WITH => {
                self.expect_exact_args(s::STD_STR_STARTS_WITH, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::StartsWith), location);
                Ok(true)
            }
            s::STD_STR_ENDS_WITH => {
                self.expect_exact_args(s::STD_STR_ENDS_WITH, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::EndsWith), location);
                Ok(true)
            }
            s::STD_STR_SUBSTRING => {
                self.expect_exact_args(s::STD_STR_SUBSTRING, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::Substring), location);
                Ok(true)
            }
            s::STD_STR_INDEX_OF => {
                self.expect_exact_args(s::STD_STR_INDEX_OF, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::IndexOf), location);
                Ok(true)
            }
            s::STD_STR_REPLACE => {
                self.expect_exact_args(s::STD_STR_REPLACE, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::Replace), location);
                Ok(true)
            }
            s::STD_STR_SPLIT => {
                self.expect_exact_args(s::STD_STR_SPLIT, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::Split), location);
                Ok(true)
            }
            s::STD_STR_JOIN => {
                self.expect_exact_args(s::STD_STR_JOIN, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::Join), location);
                Ok(true)
            }
            s::STD_STR_IS_NUMERIC => {
                self.expect_exact_args(s::STD_STR_IS_NUMERIC, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::IsNumeric), location);
                Ok(true)
            }
            s::STD_STR_REPEAT => {
                self.expect_exact_args(s::STD_STR_REPEAT, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::Repeat), location);
                Ok(true)
            }
            s::STD_STR_PAD_LEFT => {
                self.expect_exact_args(s::STD_STR_PAD_LEFT, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::PadLeft), location);
                Ok(true)
            }
            s::STD_STR_PAD_RIGHT => {
                self.expect_exact_args(s::STD_STR_PAD_RIGHT, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::PadRight), location);
                Ok(true)
            }
            s::STD_STR_PAD_CENTER => {
                self.expect_exact_args(s::STD_STR_PAD_CENTER, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::PadCenter), location);
                Ok(true)
            }
            s::STD_STR_FROM_CHAR => {
                self.expect_exact_args(s::STD_STR_FROM_CHAR, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::FromChar), location);
                Ok(true)
            }
            s::STD_STR_CHAR_AT => {
                self.expect_exact_args(s::STD_STR_CHAR_AT, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::CharAt), location);
                Ok(true)
            }
            s::STD_STR_SET_CHAR_AT => {
                self.expect_exact_args(s::STD_STR_SET_CHAR_AT, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::SetCharAt), location);
                Ok(true)
            }
            s::STD_STR_ORD => {
                self.expect_exact_args(s::STD_STR_ORD, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::Ord), location);
                Ok(true)
            }
            s::STD_STR_CHR => {
                self.expect_exact_args(s::STD_STR_CHR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::Chr), location);
                Ok(true)
            }
            s::STD_STR_INSERT => {
                self.expect_exact_args(s::STD_STR_INSERT, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::Insert), location);
                Ok(true)
            }
            s::STD_STR_DELETE => {
                self.expect_exact_args(s::STD_STR_DELETE, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::Delete), location);
                Ok(true)
            }
            s::STD_STR_REVERSE => {
                self.expect_exact_args(s::STD_STR_REVERSE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::Reverse), location);
                Ok(true)
            }
            s::STD_STR_TRIM_LEFT => {
                self.expect_exact_args(s::STD_STR_TRIM_LEFT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::TrimLeft), location);
                Ok(true)
            }
            s::STD_STR_TRIM_RIGHT => {
                self.expect_exact_args(s::STD_STR_TRIM_RIGHT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::TrimRight), location);
                Ok(true)
            }
            s::STD_STR_LAST_INDEX_OF => {
                self.expect_exact_args(s::STD_STR_LAST_INDEX_OF, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Str(StrIntrinsic::LastIndexOf), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
