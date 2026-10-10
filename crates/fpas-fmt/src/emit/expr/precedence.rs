//! Operator binding strengths and required binary operand parentheses.
//!
//! Documentation: `docs/pascal/language/basics/operators.md#operator-precedence`.

use fpas_parser::{BinaryOp, Expr, UnaryOp};

/// Binding strength shared by arithmetic negation and error propagation.
pub(super) const PREFIX_PREC: u8 = 6;

/// Returns the binding strength of each implemented binary operator.
pub(super) fn binary_prec(op: BinaryOp) -> u8 {
    match op {
        BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => 1,
        BinaryOp::Eq
        | BinaryOp::NotEq
        | BinaryOp::Lt
        | BinaryOp::Gt
        | BinaryOp::LtEq
        | BinaryOp::GtEq
        | BinaryOp::In => 3,
        BinaryOp::Add | BinaryOp::Sub => 4,
        BinaryOp::Mul | BinaryOp::RealDiv | BinaryOp::IntDiv | BinaryOp::Mod => 5,
    }
}

/// Keeps logical negation below comparisons and arithmetic negation above arithmetic.
pub(super) fn unary_prec(op: UnaryOp) -> u8 {
    match op {
        UnaryOp::Not => 2,
        UnaryOp::Negate => PREFIX_PREC,
    }
}

/// Requires parentheses for mixed logical operands, nested comparisons, and right
/// operands of left-associative arithmetic at the same binding strength.
pub(super) fn operand_prec(parent: BinaryOp, child: &Expr, right: bool) -> u8 {
    let prec = binary_prec(parent);
    if prec == 1 {
        let mixed = matches!(child, Expr::BinaryOp { op, .. }
            if binary_prec(*op) == prec && *op != parent);
        prec + u8::from(right || mixed)
    } else if prec == 3 {
        prec + 1
    } else {
        // `A - B - C` groups as `(A - B) - C`, so only the right operand needs parentheses.
        prec + u8::from(right)
    }
}

#[cfg(test)]
mod tests;
