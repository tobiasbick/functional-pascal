//! Lowers `Std.Array` calls to VM intrinsics.
//!
//! **Documentation:** `docs/pascal/std/collections/array.md` (from the repository root).

use crate::error::CompileError;
use fpas_bytecode::{ArrayIntrinsic, Intrinsic, Op, SourceLocation};
use fpas_parser::Expr;
use fpas_std::std_symbols as s;

use super::Compiler;

impl Compiler {
    pub(super) fn compile_array_call(
        &mut self,
        name: &str,
        args: &[Expr],
        location: SourceLocation,
    ) -> Result<bool, CompileError> {
        match name {
            s::STD_ARRAY_LENGTH => {
                self.expect_exact_args(s::STD_ARRAY_LENGTH, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::Length), location);
                Ok(true)
            }
            s::STD_ARRAY_SORT => {
                self.expect_exact_args(s::STD_ARRAY_SORT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::Sort), location);
                Ok(true)
            }
            s::STD_ARRAY_REVERSE => {
                self.expect_exact_args(s::STD_ARRAY_REVERSE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::Reverse), location);
                Ok(true)
            }
            s::STD_ARRAY_CONTAINS => {
                self.expect_exact_args(s::STD_ARRAY_CONTAINS, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::Contains), location);
                Ok(true)
            }
            s::STD_ARRAY_INDEX_OF => {
                self.expect_exact_args(s::STD_ARRAY_INDEX_OF, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::IndexOf), location);
                Ok(true)
            }
            s::STD_ARRAY_SLICE => {
                self.expect_exact_args(s::STD_ARRAY_SLICE, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::Slice), location);
                Ok(true)
            }
            s::STD_ARRAY_PUSH => {
                self.expect_exact_args(s::STD_ARRAY_PUSH, 2, args, location)?;
                let (depth, slot) =
                    self.mutable_array_local_ref(s::STD_ARRAY_PUSH, &args[0], location)?;
                self.compile_expr(&args[1])?;
                self.emit(Op::ArrayPushLocal(depth, slot), location);
                self.emit(Op::Unit, location);
                Ok(true)
            }
            s::STD_ARRAY_POP => {
                self.expect_exact_args(s::STD_ARRAY_POP, 1, args, location)?;
                let (depth, slot) =
                    self.mutable_array_local_ref(s::STD_ARRAY_POP, &args[0], location)?;
                self.emit(Op::ArrayPopLocal(depth, slot), location);
                Ok(true)
            }
            s::STD_ARRAY_MAP => {
                self.expect_exact_args(s::STD_ARRAY_MAP, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::Map), location);
                Ok(true)
            }
            s::STD_ARRAY_FILTER => {
                self.expect_exact_args(s::STD_ARRAY_FILTER, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::Filter), location);
                Ok(true)
            }
            s::STD_ARRAY_REDUCE => {
                self.expect_exact_args(s::STD_ARRAY_REDUCE, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::Reduce), location);
                Ok(true)
            }
            s::STD_ARRAY_CONCAT => {
                self.expect_exact_args(s::STD_ARRAY_CONCAT, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::Concat), location);
                Ok(true)
            }
            s::STD_ARRAY_FILL => {
                self.expect_exact_args(s::STD_ARRAY_FILL, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::Fill), location);
                Ok(true)
            }
            s::STD_ARRAY_FIND => {
                self.expect_exact_args(s::STD_ARRAY_FIND, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::Find), location);
                Ok(true)
            }
            s::STD_ARRAY_FIND_INDEX => {
                self.expect_exact_args(s::STD_ARRAY_FIND_INDEX, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::FindIndex), location);
                Ok(true)
            }
            s::STD_ARRAY_ANY => {
                self.expect_exact_args(s::STD_ARRAY_ANY, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::Any), location);
                Ok(true)
            }
            s::STD_ARRAY_ALL => {
                self.expect_exact_args(s::STD_ARRAY_ALL, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::All), location);
                Ok(true)
            }
            s::STD_ARRAY_FLAT_MAP => {
                self.expect_exact_args(s::STD_ARRAY_FLAT_MAP, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::FlatMap), location);
                Ok(true)
            }
            s::STD_ARRAY_FOR_EACH => {
                self.expect_exact_args(s::STD_ARRAY_FOR_EACH, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::Array(ArrayIntrinsic::ForEach), location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
