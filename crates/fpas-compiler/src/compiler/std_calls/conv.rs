//! Lowers `Std.Conv` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/conv.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_conv_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_CONV_INT_TO_STR => {
                self.expect_exact_args(s::STD_CONV_INT_TO_STR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ConvIntToStr, location);
                Ok(true)
            }
            s::STD_CONV_STR_TO_INT => {
                self.expect_exact_args(s::STD_CONV_STR_TO_INT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ConvStrToInt, location);
                Ok(true)
            }
            s::STD_CONV_REAL_TO_STR => {
                self.expect_exact_args(s::STD_CONV_REAL_TO_STR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ConvRealToStr, location);
                Ok(true)
            }
            s::STD_CONV_STR_TO_REAL => {
                self.expect_exact_args(s::STD_CONV_STR_TO_REAL, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ConvStrToReal, location);
                Ok(true)
            }
            s::STD_CONV_CHAR_TO_STR => {
                self.expect_exact_args(s::STD_CONV_CHAR_TO_STR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ConvCharToStr, location);
                Ok(true)
            }
            s::STD_CONV_INT_TO_REAL => {
                self.expect_exact_args(s::STD_CONV_INT_TO_REAL, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ConvIntToReal, location);
                Ok(true)
            }
            s::STD_CONV_BOOL_TO_STR => {
                self.expect_exact_args(s::STD_CONV_BOOL_TO_STR, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ConvBoolToStr, location);
                Ok(true)
            }
            s::STD_CONV_STR_TO_BOOL => {
                self.expect_exact_args(s::STD_CONV_STR_TO_BOOL, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ConvStrToBool, location);
                Ok(true)
            }
            s::STD_CONV_INT_TO_HEX => {
                self.expect_exact_args(s::STD_CONV_INT_TO_HEX, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::ConvIntToHex, location);
                Ok(true)
            }
            s::STD_CONV_HEX_TO_INT => {
                self.expect_exact_args(s::STD_CONV_HEX_TO_INT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ConvHexToInt, location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
