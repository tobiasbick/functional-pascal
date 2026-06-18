//! Lowers `Std.Math` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/numeric/math.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, MathIntrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_math_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_MATH_SQRT => {
                self.expect_exact_args(s::STD_MATH_SQRT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Sqrt), location);
                Ok(true)
            }
            s::STD_MATH_POW => {
                self.expect_exact_args(s::STD_MATH_POW, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Pow), location);
                Ok(true)
            }
            s::STD_MATH_FLOOR => {
                self.expect_exact_args(s::STD_MATH_FLOOR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Floor), location);
                Ok(true)
            }
            s::STD_MATH_CEIL => {
                self.expect_exact_args(s::STD_MATH_CEIL, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Ceil), location);
                Ok(true)
            }
            s::STD_MATH_ROUND => {
                self.expect_exact_args(s::STD_MATH_ROUND, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Round), location);
                Ok(true)
            }
            s::STD_MATH_SIN => {
                self.expect_exact_args(s::STD_MATH_SIN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Sin), location);
                Ok(true)
            }
            s::STD_MATH_COS => {
                self.expect_exact_args(s::STD_MATH_COS, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Cos), location);
                Ok(true)
            }
            s::STD_MATH_LOG => {
                self.expect_exact_args(s::STD_MATH_LOG, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Log), location);
                Ok(true)
            }
            s::STD_MATH_ABS => {
                self.expect_exact_args(s::STD_MATH_ABS, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Abs), location);
                Ok(true)
            }
            s::STD_MATH_MIN => {
                self.expect_exact_args(s::STD_MATH_MIN, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Min), location);
                Ok(true)
            }
            s::STD_MATH_MAX => {
                self.expect_exact_args(s::STD_MATH_MAX, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Max), location);
                Ok(true)
            }
            s::STD_MATH_TAN => {
                self.expect_exact_args(s::STD_MATH_TAN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Tan), location);
                Ok(true)
            }
            s::STD_MATH_ARC_SIN => {
                self.expect_exact_args(s::STD_MATH_ARC_SIN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::ArcSin), location);
                Ok(true)
            }
            s::STD_MATH_ARC_COS => {
                self.expect_exact_args(s::STD_MATH_ARC_COS, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::ArcCos), location);
                Ok(true)
            }
            s::STD_MATH_ARC_TAN => {
                self.expect_exact_args(s::STD_MATH_ARC_TAN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::ArcTan), location);
                Ok(true)
            }
            s::STD_MATH_ARC_TAN2 => {
                self.expect_exact_args(s::STD_MATH_ARC_TAN2, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::ArcTan2), location);
                Ok(true)
            }
            s::STD_MATH_EXP => {
                self.expect_exact_args(s::STD_MATH_EXP, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Exp), location);
                Ok(true)
            }
            s::STD_MATH_LOG10 => {
                self.expect_exact_args(s::STD_MATH_LOG10, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Log10), location);
                Ok(true)
            }
            s::STD_MATH_LOG2 => {
                self.expect_exact_args(s::STD_MATH_LOG2, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Log2), location);
                Ok(true)
            }
            s::STD_MATH_TRUNC => {
                self.expect_exact_args(s::STD_MATH_TRUNC, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Trunc), location);
                Ok(true)
            }
            s::STD_MATH_FRAC => {
                self.expect_exact_args(s::STD_MATH_FRAC, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Frac), location);
                Ok(true)
            }
            s::STD_MATH_SIGN => {
                self.expect_exact_args(s::STD_MATH_SIGN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Sign), location);
                Ok(true)
            }
            s::STD_MATH_CLAMP => {
                self.expect_exact_args(s::STD_MATH_CLAMP, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::Math(MathIntrinsic::Clamp), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
