//! Binary operators and line-breaking for long chains.

use fpas_parser::{BinaryOp, Expr};

use crate::comments::CommentMap;

use super::super::Emitter;
use super::precedence::operand_prec;

/// Breaks a binary expression while retaining its required operand parentheses.
pub(super) fn emit_binary_with_break(
    emitter: &mut Emitter,
    expr: &Expr,
    base_column: usize,
    comments: &CommentMap,
) {
    let Expr::BinaryOp {
        op, left, right, ..
    } = expr
    else {
        super::emit_expr_impl(emitter, expr, 0, false, comments);
        return;
    };
    super::emit_expr_impl(
        emitter,
        left,
        operand_prec(*op, left, false),
        false,
        comments,
    );
    let op_token = binary_op_spaced(*op).trim();
    emitter.write(" ");
    emitter.write(op_token);
    emitter.newline_to_column(base_column);
    super::emit_expr_impl(
        emitter,
        right,
        operand_prec(*op, right, true),
        false,
        comments,
    );
}

pub(super) fn binary_op_spaced(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Mul => " * ",
        BinaryOp::RealDiv => " / ",
        BinaryOp::IntDiv => " div ",
        BinaryOp::Mod => " mod ",
        BinaryOp::And => " and ",
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
