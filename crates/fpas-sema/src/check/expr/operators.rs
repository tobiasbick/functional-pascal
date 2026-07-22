use super::super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::{BinaryOp, Expr, UnaryOp};

fn binary_op_symbol(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::RealDiv => "/",
        BinaryOp::IntDiv => "div",
        BinaryOp::Mod => "mod",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
        BinaryOp::Xor => "xor",
        BinaryOp::Shl => "shl",
        BinaryOp::Shr => "shr",
        BinaryOp::Eq => "=",
        BinaryOp::NotEq => "<>",
        BinaryOp::Lt => "<",
        BinaryOp::Gt => ">",
        BinaryOp::LtEq => "<=",
        BinaryOp::GtEq => ">=",
        BinaryOp::In => "in",
    }
}

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
                    } else if *left == Ty::Real || *right == Ty::Real || op == BinaryOp::RealDiv {
                        Ty::Real
                    } else {
                        Ty::Integer
                    }
                } else if op == BinaryOp::Add
                    && matches!(left, Ty::String)
                    && matches!(right, Ty::String)
                {
                    Ty::String
                } else {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        format!("Operator `{}` requires numeric operands", binary_op_symbol(op)),
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
                        format!(
                            "Operator `{}` requires integer operands",
                            binary_op_symbol(op)
                        ),
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
                        format!(
                            "Operator `{}` requires boolean or integer operands",
                            binary_op_symbol(op)
                        ),
                        "Both sides must be the same type (boolean or integer).",
                        span,
                    );
                    Ty::Error
                }
            }

            BinaryOp::Eq | BinaryOp::NotEq => {
                let supports_equality = (left.is_comparable() || left.is_ordinal())
                    && (right.is_comparable() || right.is_ordinal())
                    || matches!(
                        (left, right),
                        (Ty::Option(_), Ty::Option(_)) | (Ty::Result(..), Ty::Result(..))
                    );
                if !left.compatible_with(right) || !supports_equality {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        "Equality requires compatible scalar, option, or result operands",
                        "Compare integer, real, boolean, string, enum, option, or result values; compare record fields explicitly.",
                        span,
                    );
                    return Ty::Error;
                }
                Ty::Boolean
            }

            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => {
                if (left.is_comparable() && right.is_comparable() && left.compatible_with(right))
                    || (left.is_numeric() && right.is_numeric())
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

            BinaryOp::In => match right {
                Ty::Array(element_ty) if left.compatible_with(element_ty) => Ty::Boolean,
                Ty::Dict(key_ty, _) if left.compatible_with(key_ty) => Ty::Boolean,
                Ty::String if matches!(left, Ty::String) => Ty::Boolean,
                _ => {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        "Operator `in` requires a value and a compatible array, dict, or string",
                        "Use `Item in Array`, `Key in Dict`, or `Substring in String`.",
                        span,
                    );
                    Ty::Error
                }
            },
        }
    }
}
