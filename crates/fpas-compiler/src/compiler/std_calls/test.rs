//! Lowers `Std.Test` calls to runtime intrinsics.
//!
//! **Documentation:** `docs/pascal/std/test.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, SourceLocation, TestIntrinsic};
use fpas_parser::Expr;
use fpas_sema::Ty;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    /// Compile a `Std.Test` call into a runtime intrinsic when `name` belongs to the unit.
    pub(super) fn compile_test_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_TEST_ASSERT_TRUE => {
                self.expect_exact_args(s::STD_TEST_ASSERT_TRUE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Test(TestIntrinsic::AssertTrue), location);
                Ok(true)
            }
            s::STD_TEST_ASSERT_FALSE => {
                self.expect_exact_args(s::STD_TEST_ASSERT_FALSE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Test(TestIntrinsic::AssertFalse), location);
                Ok(true)
            }
            s::STD_TEST_ASSERT_EQUALS => {
                self.expect_exact_args(s::STD_TEST_ASSERT_EQUALS, 2, args, location)?;
                let operand_ty = self.ty_of(&args[0]);
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                let intrinsic = match operand_ty {
                    Ty::Integer => TestIntrinsic::AssertEqualsInteger,
                    Ty::Boolean => TestIntrinsic::AssertEqualsBoolean,
                    Ty::String => TestIntrinsic::AssertEqualsString,
                    Ty::Real => TestIntrinsic::AssertEqualsReal,
                    _ => TestIntrinsic::AssertEqualsInteger,
                };
                self.emit_intrinsic_unit(Intrinsic::Test(intrinsic), location);
                Ok(true)
            }
            s::STD_TEST_FAIL => {
                self.expect_exact_args(s::STD_TEST_FAIL, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Test(TestIntrinsic::Fail), location);
                Ok(true)
            }
            s::STD_TEST_SKIP => {
                self.expect_exact_args(s::STD_TEST_SKIP, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Test(TestIntrinsic::Skip), location);
                Ok(true)
            }
            s::STD_TEST_ASSERT_SCREEN_LINE => {
                self.expect_exact_args(s::STD_TEST_ASSERT_SCREEN_LINE, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic_unit(
                    Intrinsic::Test(TestIntrinsic::AssertScreenLine),
                    location,
                );
                Ok(true)
            }
            s::STD_TEST_ASSERT_SCREEN_CELL => {
                self.expect_exact_args(s::STD_TEST_ASSERT_SCREEN_CELL, 5, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(
                    Intrinsic::Test(TestIntrinsic::AssertScreenCell),
                    location,
                );
                Ok(true)
            }
            s::STD_TEST_ASSERT_VIEW_RECT => {
                self.expect_exact_args(s::STD_TEST_ASSERT_VIEW_RECT, 6, args, location)?;
                for arg in args {
                    self.compile_expr(arg)?;
                }
                self.emit_intrinsic_unit(Intrinsic::Test(TestIntrinsic::AssertViewRect), location);
                Ok(true)
            }
            s::STD_TEST_PUSH_READLN => {
                self.expect_exact_args(s::STD_TEST_PUSH_READLN, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic_unit(Intrinsic::Test(TestIntrinsic::PushReadLn), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
