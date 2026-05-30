//! Lowers `Std.Random` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/random.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, RandomIntrinsic, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_random_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_RANDOM_RANDOM => {
                self.expect_exact_args(s::STD_RANDOM_RANDOM, 0, args, location)?;
                self.emit_intrinsic(Intrinsic::Random(RandomIntrinsic::Random), location);
                Ok(true)
            }
            s::STD_RANDOM_RANDOM_INT => {
                self.expect_exact_args(s::STD_RANDOM_RANDOM_INT, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Random(RandomIntrinsic::RandomInt), location);
                Ok(true)
            }
            s::STD_RANDOM_RANDOMIZE => {
                self.expect_exact_args(s::STD_RANDOM_RANDOMIZE, 0, args, location)?;
                self.emit_intrinsic(Intrinsic::Random(RandomIntrinsic::Randomize), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
