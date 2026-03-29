use super::super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::{BinaryOp, Expr, UnaryOp};

impl Checker {
    pub(super) fn check_unary_expr(&mut self, op: UnaryOp, operand: &Expr, span: Span) -> Ty {
        let operand_ty = self.check_expr(operand);

        match op {
            UnaryOp::Negate => {
                if operand_ty.is_numeric() || operand_ty.is_error() {
                    operand_ty
                } else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        "Unary `-` requires a numeric operand",
                        "Use integer or real values, or a generic type with Numeric constraint.",
                        span,
                    );
                    Ty::Error
                }
            }
            UnaryOp::Not => {
                if operand_ty.compatible_with(&Ty::Boolean)
                    || operand_ty.compatible_with(&Ty::Integer)
                {
                    operand_ty
                } else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        "`not` requires a boolean or integer operand",
                        "Use boolean or integer values.",
                        span,
                    );
                    Ty::Error
                }
            }
        }
    }

    pub(super) fn check_binary_expr(
        &mut self,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
        span: Span,
    ) -> Ty {
        let left_ty = self.check_expr(left);
        let right_ty = self.check_expr(right);
        self.check_binary_op(op, &left_ty, &right_ty, span)
    }

    fn check_binary_op(&mut self, op: BinaryOp, left: &Ty, right: &Ty, span: Span) -> Ty {
        if left.is_error() || right.is_error() {
            return Ty::Error;
        }

        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::RealDiv => {
                if left.is_numeric() && right.is_numeric() {
                    // When both sides are GenericParam, return the left param type.
                    if matches!(left, Ty::GenericParam(..)) {
                        left.clone()
                    } else if *left == Ty::Real || *right == Ty::Real {
                        Ty::Real
                    } else {
                        Ty::Integer
                    }
                } else if op == BinaryOp::Add
                    && matches!(left, Ty::String | Ty::Char)
                    && matches!(right, Ty::String | Ty::Char)
                {
                    Ty::String
                } else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Operator `{op:?}` requires numeric operands"),
                        "Both sides must be integer or real, or a generic type with Numeric constraint.",
                        span,
                    );
                    Ty::Error
                }
            }

            BinaryOp::IntDiv | BinaryOp::Mod | BinaryOp::Shl | BinaryOp::Shr => {
                if *left == Ty::Integer && *right == Ty::Integer {
                    Ty::Integer
                } else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Operator `{op:?}` requires integer operands"),
                        "Both sides must be integer.",
                        span,
                    );
                    Ty::Error
                }
            }

            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
                if *left == Ty::Boolean && *right == Ty::Boolean {
                    Ty::Boolean
                } else if *left == Ty::Integer && *right == Ty::Integer {
                    Ty::Integer
                } else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Operator `{op:?}` requires boolean or integer operands"),
                        "Both sides must be the same type (boolean or integer).",
                        span,
                    );
                    Ty::Error
                }
            }

            BinaryOp::Eq | BinaryOp::NotEq => {
                if !left.compatible_with(right) {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        "Comparison operands must be the same type",
                        "Both sides of `=` or `<>` must match.",
                        span,
                    );
                }
                Ty::Boolean
            }

            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => {
                if left.is_comparable() && right.is_comparable() && left.compatible_with(right) {
                    Ty::Boolean
                } else if (left.is_ordinal() && right.is_ordinal() && left.compatible_with(right))
                    || (left.is_numeric() && right.is_numeric())
                    || (*left == Ty::String && *right == Ty::String)
                {
                    Ty::Boolean
                } else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        "Comparison requires compatible ordinal, numeric, or string operands",
                        "Both sides must be comparable.",
                        span,
                    );
                    Ty::Error
                }
            }
        }
    }
}
