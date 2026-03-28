use crate::error::CompileError;
use fpas_bytecode::{Intrinsic, Op, SourceLocation};
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
                self.emit_intrinsic(Intrinsic::ArrayLength, location);
                Ok(true)
            }
            s::STD_ARRAY_SORT => {
                self.expect_exact_args(s::STD_ARRAY_SORT, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ArraySort, location);
                Ok(true)
            }
            s::STD_ARRAY_REVERSE => {
                self.expect_exact_args(s::STD_ARRAY_REVERSE, 1, args, location)?;
                self.compile_expr(&args[0])?;
                self.emit_intrinsic(Intrinsic::ArrayReverse, location);
                Ok(true)
            }
            s::STD_ARRAY_CONTAINS => {
                self.expect_exact_args(s::STD_ARRAY_CONTAINS, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::ArrayContains, location);
                Ok(true)
            }
            s::STD_ARRAY_INDEX_OF => {
                self.expect_exact_args(s::STD_ARRAY_INDEX_OF, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::ArrayIndexOf, location);
                Ok(true)
            }
            s::STD_ARRAY_SLICE => {
                self.expect_exact_args(s::STD_ARRAY_SLICE, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::ArraySlice, location);
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
                self.emit_intrinsic(Intrinsic::ArrayMap, location);
                Ok(true)
            }
            s::STD_ARRAY_FILTER => {
                self.expect_exact_args(s::STD_ARRAY_FILTER, 2, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.emit_intrinsic(Intrinsic::ArrayFilter, location);
                Ok(true)
            }
            s::STD_ARRAY_REDUCE => {
                self.expect_exact_args(s::STD_ARRAY_REDUCE, 3, args, location)?;
                self.compile_expr(&args[0])?;
                self.compile_expr(&args[1])?;
                self.compile_expr(&args[2])?;
                self.emit_intrinsic(Intrinsic::ArrayReduce, location);
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
