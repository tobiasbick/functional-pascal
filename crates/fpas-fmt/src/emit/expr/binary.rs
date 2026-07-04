//! Binary operators and line-breaking for long chains.

use fpas_parser::{BinaryOp, Expr};

use super::super::Emitter;

pub(super) fn emit_binary_with_break(emitter: &mut Emitter, expr: &Expr, base_column: usize) {
    let Expr::BinaryOp {
        op, left, right, ..
    } = expr
    else {
        super::emit_expr_impl(emitter, expr, 0, false);
        return;
    };
    let prec = binary_prec(*op);
    super::emit_expr_impl(emitter, left, prec + 1, false);
    let op_token = binary_op_spaced(*op).trim();
    emitter.write(" ");
    emitter.write(op_token);
    emitter.newline_to_column(base_column);
    super::emit_expr_impl(emitter, right, prec, false);
}
pub(super) fn binary_prec(op: BinaryOp) -> u8 {
    match op {
        BinaryOp::Mul
        | BinaryOp::RealDiv
        | BinaryOp::IntDiv
        | BinaryOp::Mod
        | BinaryOp::And
        | BinaryOp::Shl
        | BinaryOp::Shr => 3,
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Or | BinaryOp::Xor => 2,
        BinaryOp::Eq
        | BinaryOp::NotEq
        | BinaryOp::Lt
        | BinaryOp::Gt
        | BinaryOp::LtEq
        | BinaryOp::GtEq
        | BinaryOp::In => 1,
    }
}

pub(super) fn binary_op_spaced(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Mul => " * ",
        BinaryOp::RealDiv => " / ",
        BinaryOp::IntDiv => " div ",
        BinaryOp::Mod => " mod ",
        BinaryOp::And => " and ",
        BinaryOp::Shl => " shl ",
        BinaryOp::Shr => " shr ",
        BinaryOp::Add => " + ",
        BinaryOp::Sub => " - ",
        BinaryOp::Or => " or ",
        BinaryOp::Xor => " xor ",
        BinaryOp::Eq => " = ",
        BinaryOp::NotEq => " <> ",
        BinaryOp::Lt => " < ",
        BinaryOp::Gt => " > ",
        BinaryOp::LtEq => " <= ",
        BinaryOp::GtEq => " >= ",
        BinaryOp::In => " in ",
    }
}
