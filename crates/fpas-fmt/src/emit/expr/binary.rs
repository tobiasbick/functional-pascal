//! Binary operators and line-breaking for long chains.

use fpas_parser::{BinaryOp, Expr};

use crate::comments::CommentMap;

use super::super::Emitter;

/// Wraps a binary expression without changing its operand grouping.
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
    let prec = binary_prec(*op);
    super::emit_expr_impl(emitter, left, left_precedence(*op, left), false, comments);
    let op_token = binary_op_spaced(*op).trim();
    emitter.write(" ");
    emitter.write(op_token);
    emitter.newline_to_column(base_column);
    super::emit_expr_impl(emitter, right, prec + 1, false, comments);
}
/// Returns the precedence defined by `docs/pascal/language/basics/operators.md`.
pub(super) fn binary_prec(op: BinaryOp) -> u8 {
    match op {
        BinaryOp::Mul | BinaryOp::RealDiv | BinaryOp::IntDiv | BinaryOp::Mod => 5,
        BinaryOp::Add | BinaryOp::Sub => 4,
        BinaryOp::Eq
        | BinaryOp::NotEq
        | BinaryOp::Lt
        | BinaryOp::Gt
        | BinaryOp::LtEq
        | BinaryOp::GtEq
        | BinaryOp::In => 3,
        BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => 1,
    }
}

/// Returns the canonical infix operator spelling with surrounding spaces.
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

/// Protects non-associative comparisons and differently grouped logical operators.
pub(super) fn left_precedence(op: BinaryOp, left: &Expr) -> u8 {
    let prec = binary_prec(op);
    if prec == 3 || (prec == 1 && matches!(left, Expr::BinaryOp { op: child, .. } if *child != op))
    {
        prec + 1
    } else {
        prec
    }
}
