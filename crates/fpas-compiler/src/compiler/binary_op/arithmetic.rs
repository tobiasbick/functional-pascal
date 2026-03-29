use crate::error::CompileError;
use fpas_bytecode::{Op, SourceLocation};
use fpas_parser::{BinaryOp, Expr};
use fpas_sema::Ty;

use super::super::Compiler;

impl Compiler {
    pub(super) fn compile_add(
        &mut self,
        left: &Expr,
        right: &Expr,
        operand_types: (&Ty, &Ty),
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let (lt, rt) = operand_types;
        if matches!((lt, rt), (Ty::String | Ty::Char, Ty::String | Ty::Char)) {
            self.emit_string_binary(left, right, operand_types, Op::ConcatStr, location)
        } else if is_generic_param(lt) || is_generic_param(rt) {
            self.compile_direct_binary(Op::AddDyn, left, right, location)
        } else if lt.is_numeric() && rt.is_numeric() {
            self.emit_numeric_binary(
                Op::AddInt,
                Op::AddReal,
                left,
                right,
                operand_types,
                location,
            )
        } else {
            unreachable!("semantic analysis should reject invalid `+` operands");
        }
    }

    pub(super) fn compile_numeric_arithmetic(
        &mut self,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
        operand_types: (&Ty, &Ty),
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let (lt, rt) = operand_types;
        if is_generic_param(lt) || is_generic_param(rt) {
            let dyn_op = match op {
                BinaryOp::Sub => Op::SubDyn,
                BinaryOp::Mul => Op::MulDyn,
                _ => unreachable!("only subtraction and multiplication reach this helper"),
            };
            return self.compile_direct_binary(dyn_op, left, right, location);
        }
        let (int_op, real_op) = match op {
            BinaryOp::Sub => (Op::SubInt, Op::SubReal),
            BinaryOp::Mul => (Op::MulInt, Op::MulReal),
            _ => unreachable!("only subtraction and multiplication reach this helper"),
        };
        self.emit_numeric_binary(int_op, real_op, left, right, operand_types, location)
    }

    pub(super) fn compile_real_div(
        &mut self,
        left: &Expr,
        right: &Expr,
        operand_types: (&Ty, &Ty),
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let (lt, rt) = operand_types;
        if is_generic_param(lt) || is_generic_param(rt) {
            return self.compile_direct_binary(Op::DivDyn, left, right, location);
        }
        self.compile_expr(left)?;
        self.maybe_int_to_real_for_ty(lt, location);
        self.compile_expr(right)?;
        self.maybe_int_to_real_for_ty(rt, location);
        self.emit(Op::DivReal, location);
        Ok(())
    }

    pub(super) fn compile_direct_binary(
        &mut self,
        op: Op,
        left: &Expr,
        right: &Expr,
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        self.compile_expr(left)?;
        self.compile_expr(right)?;
        self.emit(op, location);
        Ok(())
    }
}

fn is_generic_param(ty: &Ty) -> bool {
    matches!(ty, Ty::GenericParam(..))
}
