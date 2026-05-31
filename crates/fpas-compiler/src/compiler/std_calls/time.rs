//! Lowers `Std.Time` calls to runtime intrinsics.
//!
//! **Documentation:** `docs/pascal/std/time.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TimeIntrinsic};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    /// Compile a `Std.Time` call into a runtime intrinsic when `name` belongs to the unit.
    pub(super) fn compile_time_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_TIME_TIMESTAMP_MILLIS => {
                self.expect_zero_args(s::STD_TIME_TIMESTAMP_MILLIS, args, location)?;
                self.emit_intrinsic(Intrinsic::Time(TimeIntrinsic::TimestampMillis), location);
                Ok(true)
            }
            s::STD_TIME_MONOTONIC_MILLIS => {
                self.expect_zero_args(s::STD_TIME_MONOTONIC_MILLIS, args, location)?;
                self.emit_intrinsic(Intrinsic::Time(TimeIntrinsic::MonotonicMillis), location);
                Ok(true)
            }
            s::STD_TIME_ELAPSED_MILLIS => {
                self.expect_exact_args(s::STD_TIME_ELAPSED_MILLIS, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Time(TimeIntrinsic::ElapsedMillis), location);
                Ok(true)
            }
            s::STD_TIME_SLEEP => {
                self.expect_exact_args(s::STD_TIME_SLEEP, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Time(TimeIntrinsic::Sleep), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
