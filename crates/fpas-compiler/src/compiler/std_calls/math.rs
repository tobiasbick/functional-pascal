use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation};
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
                self.emit_intrinsic(Intrinsic::MathSqrt, location);
                Ok(true)
            }
            s::STD_MATH_POW => {
                self.expect_exact_args(s::STD_MATH_POW, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::MathPow, location);
                Ok(true)
            }
            s::STD_MATH_FLOOR => {
                self.expect_exact_args(s::STD_MATH_FLOOR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathFloor, location);
                Ok(true)
            }
            s::STD_MATH_CEIL => {
                self.expect_exact_args(s::STD_MATH_CEIL, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathCeil, location);
                Ok(true)
            }
            s::STD_MATH_ROUND => {
                self.expect_exact_args(s::STD_MATH_ROUND, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathRound, location);
                Ok(true)
            }
            s::STD_MATH_SIN => {
                self.expect_exact_args(s::STD_MATH_SIN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathSin, location);
                Ok(true)
            }
            s::STD_MATH_COS => {
                self.expect_exact_args(s::STD_MATH_COS, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathCos, location);
                Ok(true)
            }
            s::STD_MATH_LOG => {
                self.expect_exact_args(s::STD_MATH_LOG, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathLog, location);
                Ok(true)
            }
            s::STD_MATH_ABS => {
                self.expect_exact_args(s::STD_MATH_ABS, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathAbs, location);
                Ok(true)
            }
            s::STD_MATH_MIN => {
                self.expect_exact_args(s::STD_MATH_MIN, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::MathMin, location);
                Ok(true)
            }
            s::STD_MATH_MAX => {
                self.expect_exact_args(s::STD_MATH_MAX, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::MathMax, location);
                Ok(true)
            }
            s::STD_MATH_TAN => {
                self.expect_exact_args(s::STD_MATH_TAN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathTan, location);
                Ok(true)
            }
            s::STD_MATH_ARC_SIN => {
                self.expect_exact_args(s::STD_MATH_ARC_SIN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathArcSin, location);
                Ok(true)
            }
            s::STD_MATH_ARC_COS => {
                self.expect_exact_args(s::STD_MATH_ARC_COS, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathArcCos, location);
                Ok(true)
            }
            s::STD_MATH_ARC_TAN => {
                self.expect_exact_args(s::STD_MATH_ARC_TAN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathArcTan, location);
                Ok(true)
            }
            s::STD_MATH_ARC_TAN2 => {
                self.expect_exact_args(s::STD_MATH_ARC_TAN2, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::MathArcTan2, location);
                Ok(true)
            }
            s::STD_MATH_EXP => {
                self.expect_exact_args(s::STD_MATH_EXP, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathExp, location);
                Ok(true)
            }
            s::STD_MATH_LOG10 => {
                self.expect_exact_args(s::STD_MATH_LOG10, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathLog10, location);
                Ok(true)
            }
            s::STD_MATH_LOG2 => {
                self.expect_exact_args(s::STD_MATH_LOG2, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathLog2, location);
                Ok(true)
            }
            s::STD_MATH_TRUNC => {
                self.expect_exact_args(s::STD_MATH_TRUNC, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathTrunc, location);
                Ok(true)
            }
            s::STD_MATH_FRAC => {
                self.expect_exact_args(s::STD_MATH_FRAC, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathFrac, location);
                Ok(true)
            }
            s::STD_MATH_SIGN => {
                self.expect_exact_args(s::STD_MATH_SIGN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::MathSign, location);
                Ok(true)
            }
            s::STD_MATH_CLAMP => {
                self.expect_exact_args(s::STD_MATH_CLAMP, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::MathClamp, location);
                Ok(true)
            }
            s::STD_MATH_RANDOM => {
                self.expect_exact_args(s::STD_MATH_RANDOM, 0, args, location)?;
                self.emit_intrinsic(Intrinsic::MathRandom, location);
                Ok(true)
            }
            s::STD_MATH_RANDOM_INT => {
                self.expect_exact_args(s::STD_MATH_RANDOM_INT, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::MathRandomInt, location);
                Ok(true)
            }
            s::STD_MATH_RANDOMIZE => {
                self.expect_exact_args(s::STD_MATH_RANDOMIZE, 0, args, location)?;
                self.emit_intrinsic(Intrinsic::MathRandomize, location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
