use crate::error::CompileError;
use fpas_bytecode::{Op, SourceLocation};
use fpas_parser::{BinaryOp, Expr};
use fpas_sema::Ty;

use super::super::Compiler;

impl Compiler {
    pub(super) fn compile_equality(
        &mut self,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
        operand_types: (&Ty, &Ty),
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let (lt, rt) = operand_types;

        if is_generic_param(lt) || is_generic_param(rt) {
            return self.compile_direct_binary(
                if op == BinaryOp::Eq {
                    Op::EqDyn
                } else {
                    Op::NeqDyn
                },
                left,
                right,
                location,
            );
        }

        if *lt == Ty::Real || *rt == Ty::Real {
            return self.emit_numeric_binary(
                if op == BinaryOp::Eq {
                    Op::EqInt
                } else {
                    Op::NeqInt
                },
                if op == BinaryOp::Eq {
                    Op::EqReal
                } else {
                    Op::NeqReal
                },
                left,
                right,
                operand_types,
                location,
            );
        }

        if matches!((lt, rt), (Ty::Boolean, Ty::Boolean)) {
            return self.compile_direct_binary(
                if op == BinaryOp::Eq {
                    Op::EqBool
                } else {
                    Op::NeqBool
                },
                left,
                right,
                location,
            );
        }

        if matches!((lt, rt), (Ty::String | Ty::Char, Ty::String | Ty::Char)) {
            return self.emit_string_binary(
                left,
                right,
                operand_types,
                if op == BinaryOp::Eq {
                    Op::EqStr
                } else {
                    Op::NeqStr
                },
                location,
            );
        }

        self.compile_direct_binary(
            if op == BinaryOp::Eq {
                Op::EqInt
            } else {
                Op::NeqInt
            },
            left,
            right,
            location,
        )
    }

    pub(super) fn compile_ordering(
        &mut self,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
        operand_types: (&Ty, &Ty),
        location: SourceLocation,
    ) -> Result<(), CompileError> {
        let (lt, rt) = operand_types;

        if is_generic_param(lt) || is_generic_param(rt) {
            return self.compile_direct_binary(ordering_dyn_op(op), left, right, location);
        }

        if matches!((lt, rt), (Ty::String | Ty::Char, Ty::String | Ty::Char)) {
            return self.emit_string_binary(
                left,
                right,
                operand_types,
                ordering_string_op(op),
                location,
            );
        }

        if lt.is_numeric() && rt.is_numeric() && (*lt == Ty::Real || *rt == Ty::Real) {
            return self.emit_numeric_binary(
                ordering_int_op(op),
                ordering_real_op(op),
                left,
                right,
                operand_types,
                location,
            );
        }

        self.compile_direct_binary(ordering_int_op(op), left, right, location)
    }
}

fn is_generic_param(ty: &Ty) -> bool {
    matches!(ty, Ty::GenericParam(..))
}

fn ordering_dyn_op(op: BinaryOp) -> Op {
    match op {
        BinaryOp::Lt => Op::LtDyn,
        BinaryOp::Gt => Op::GtDyn,
        BinaryOp::LtEq => Op::LeDyn,
        BinaryOp::GtEq => Op::GeDyn,
        _ => unreachable!("only ordering operators reach dynamic ordering"),
    }
}

fn ordering_string_op(op: BinaryOp) -> Op {
    match op {
        BinaryOp::Lt => Op::LtStr,
        BinaryOp::Gt => Op::GtStr,
        BinaryOp::LtEq => Op::LeStr,
        BinaryOp::GtEq => Op::GeStr,
        _ => unreachable!("only ordering operators reach string ordering"),
    }
}

fn ordering_int_op(op: BinaryOp) -> Op {
    match op {
        BinaryOp::Lt => Op::LtInt,
        BinaryOp::Gt => Op::GtInt,
        BinaryOp::LtEq => Op::LeInt,
        BinaryOp::GtEq => Op::GeInt,
        _ => unreachable!("only ordering operators reach integer ordering"),
    }
}

fn ordering_real_op(op: BinaryOp) -> Op {
    match op {
        BinaryOp::Lt => Op::LtReal,
        BinaryOp::Gt => Op::GtReal,
        BinaryOp::LtEq => Op::LeReal,
        BinaryOp::GtEq => Op::GeReal,
        _ => unreachable!("only ordering operators reach real ordering"),
    }
}
