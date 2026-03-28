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
            _ => Ok(false),
        }
    }
}
