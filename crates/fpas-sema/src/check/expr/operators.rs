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
    /// Checks arithmetic negation or Boolean-only logical negation.
    pub(super) fn check_unary_expr(&mut self, op: UnaryOp, operand: &Expr, span: Span) -> Ty {
        let operand_ty = self.check_expr(operand);
        let operand_ty = self.resolve_visible_type(&operand_ty);
        if matches!(operand_ty, Ty::Subrange(_)) && op == UnaryOp::Negate {
            return Ty::Integer;
        }
        if let Ty::Distinct(distinct) = self.resolve_visible_type(&operand_ty) {
            let symbol = match op {
                UnaryOp::Negate => "-",
                UnaryOp::Not => "not",
            };
            self.report_distinct_operator(symbol, op == UnaryOp::Negate, &distinct, span);
            return Ty::Error;
        }

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
            UnaryOp::Not => self.check_boolean_not(&operand_ty, span),
        }
    }

    /// Checks binary operands with resolved visible types and Boolean-only logical operators.
    pub(super) fn check_binary_expr(
        &mut self,
        expression: &Expr,
        op: BinaryOp,
        left: &Expr,
        right: &Expr,
        span: Span,
    ) -> Ty {
        let type_membership = op == BinaryOp::In && self.is_type_designator(right);
        // Membership owns the diagnostic for a type name supplied on the left.
        let left_ty = if type_membership && self.is_type_designator(left) {
            self.check_type_operand(left)
        } else {
            self.check_expr(left)
        };
        let left_ty = self.resolve_visible_type(&left_ty);
        let right_ty = if type_membership {
            self.check_type_operand(right)
        } else {
            self.check_expr(right)
        };
        let right_ty = self.resolve_visible_type(&right_ty);
        if op == BinaryOp::In
            && let Some(result) = self.check_subrange_membership(expression, &left_ty, &right_ty)
        {
            return result;
        }
        self.check_binary_op(op, &left_ty, &right_ty, span)
    }

    fn check_binary_op(&mut self, op: BinaryOp, left: &Ty, right: &Ty, span: Span) -> Ty {
        if left.is_error() || right.is_error() {
            return Ty::Error;
        }
        if matches!(left, Ty::Distinct(_)) || matches!(right, Ty::Distinct(_)) {
            return self.check_distinct_binary_op(op, binary_op_symbol(op), left, right, span);
        }

        let left_integer = Ty::Integer;
        let right_integer = Ty::Integer;
        let left = if matches!(left, Ty::Subrange(_)) && op != BinaryOp::In {
            &left_integer
        } else {
            left
        };
        let right = if matches!(right, Ty::Subrange(_)) && op != BinaryOp::In {
            &right_integer
        } else {
            right
        };

        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::RealDiv => {
                if left.is_numeric() && right.is_numeric() {
                    if !self.check_same_numeric_operands(left, right, span) {
                        return Ty::Error;
                    }
                    if op == BinaryOp::RealDiv {
                        Ty::Real
                    } else {
                        left.clone()
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

            BinaryOp::IntDiv | BinaryOp::Mod => {
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
                self.check_boolean_binary(op, left, right, span)
            }

            BinaryOp::Eq | BinaryOp::NotEq => {
                if (left.is_numeric() || right.is_numeric())
                    && !self.check_same_numeric_operands(left, right, span)
                {
                    return Ty::Error;
                }
                let supports_equality =
                    self.supports_equality(left) && self.supports_equality(right);
                if !left.compatible_with(right) || !supports_equality {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        "Equality requires compatible operands whose values all compare",
                        "Compare scalars, strings, enums, distinct types, or options, results, records, and enums whose payloads and fields all compare; arrays, dictionaries, callables, tasks, and channels do not compare.",
                        span,
                    );
                    return Ty::Error;
                }
                Ty::Boolean
            }

            BinaryOp::Lt | BinaryOp::Gt | BinaryOp::LtEq | BinaryOp::GtEq => {
                if (left.is_numeric() || right.is_numeric())
                    && !self.check_same_numeric_operands(left, right, span)
                {
                    return Ty::Error;
                }
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
                Ty::Array(element_ty) if element_ty.assignment_compatible_with(left) => Ty::Boolean,
                Ty::Dict(key_ty, _) if key_ty.assignment_compatible_with(left) => Ty::Boolean,
                Ty::String if matches!(left, Ty::String) => Ty::Boolean,
                _ => {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        "Operator `in` requires a compatible array, dict, string, or subrange type on the right",
                        "Use `Item in Array`, `Key in Dict`, `Substring in String`, or `Value in Percent` where `type Percent = 0..100;`.",
                        span,
                    );
                    Ty::Error
                }
            },
        }
    }

    fn check_same_numeric_operands(&mut self, left: &Ty, right: &Ty, span: Span) -> bool {
        // Generic wildcards do not permit operations on unrelated numeric types.
        let same = match (left, right) {
            (Ty::GenericParam(left, _), Ty::GenericParam(right, _)) => {
                left.eq_ignore_ascii_case(right)
            }
            _ => left == right,
        };
        if same {
            return true;
        }
        self.error_with_code(
            SEMA_TYPE_MISMATCH,
            "Numeric operands must have the same type",
            "Import Std.Conv and convert integer explicitly, for example IntToReal(1) + 0.5; generic operands must use the same Numeric type parameter.",
            span,
        );
        false
    }
}
