//! Operators on distinct types: inherited comparisons and membership, nothing else.
//!
//! **Documentation:** `docs/pascal/language/types/distinct-types.md`

use super::Checker;
use crate::types::{DistinctTy, Ty};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::BinaryOp;

impl Checker {
    /// Checks a binary operator with at least one distinct operand.
    pub(super) fn check_distinct_binary_op(
        &mut self,
        op: BinaryOp,
        symbol: &str,
        left: &Ty,
        right: &Ty,
        span: Span,
    ) -> Ty {
        match op {
            BinaryOp::Eq
            | BinaryOp::NotEq
            | BinaryOp::Lt
            | BinaryOp::Gt
            | BinaryOp::LtEq
            | BinaryOp::GtEq => self.check_distinct_comparison(symbol, left, right, span),
            BinaryOp::In => self.check_distinct_membership(left, right, span),
            BinaryOp::Add
            | BinaryOp::Sub
            | BinaryOp::Mul
            | BinaryOp::RealDiv
            | BinaryOp::IntDiv
            | BinaryOp::Mod => self.reject_distinct_operator(symbol, true, left, right, span),
            BinaryOp::And | BinaryOp::Or | BinaryOp::Xor => {
                self.reject_distinct_operator(symbol, false, left, right, span)
            }
        }
    }

    /// Same-type operands inherit `=`, `<>`, `<`, `>`, `<=`, and `>=` from the scalar underlying type.
    fn check_distinct_comparison(&mut self, symbol: &str, left: &Ty, right: &Ty, span: Span) -> Ty {
        match (left, right) {
            (Ty::Distinct(left), Ty::Distinct(right)) if left.same_declaration(right) => {
                Ty::Boolean
            }
            (Ty::Distinct(left), Ty::Distinct(right)) => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Cannot compare distinct type `{}` with distinct type `{}`",
                        left.name, right.name
                    ),
                    format!(
                        "Distinct types are not interchangeable. Compare the underlying values explicitly, for example `{}(Left) {symbol} {}(Right)`.",
                        left.underlying, right.underlying
                    ),
                    span,
                );
                Ty::Error
            }
            (Ty::Distinct(distinct), other) | (other, Ty::Distinct(distinct)) => {
                let hint = if distinct.underlying.assignment_compatible_with(other) {
                    format!(
                        "Distinct values are not converted implicitly. Wrap the other operand, for example `Value {symbol} {}(42)`, or unwrap with `{}(Value)`.",
                        distinct.name, distinct.underlying
                    )
                } else {
                    format!(
                        "Compare two values of type `{}`, or unwrap with `{}(Value)`.",
                        distinct.name, distinct.underlying
                    )
                };
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Cannot compare distinct type `{}` with `{other}`",
                        distinct.name
                    ),
                    hint,
                    span,
                );
                Ty::Error
            }
            _ => Ty::Error,
        }
    }

    /// `Value in Items` and `Key in Lookup` follow equality; substring tests are not inherited.
    fn check_distinct_membership(&mut self, left: &Ty, right: &Ty, span: Span) -> Ty {
        let element = match self.resolve_visible_type(right) {
            Ty::Array(element) => Some(*element),
            Ty::Dict(key, _) => Some(*key),
            _ => None,
        };
        let element = element.map(|element| self.resolve_visible_type(&element));
        match (left, element) {
            (Ty::Distinct(value), Some(Ty::Distinct(element)))
                if value.same_declaration(&element) =>
            {
                Ty::Boolean
            }
            (Ty::Distinct(value), Some(element)) => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Operator `in` requires `{}` elements or keys, found `{element}`",
                        value.name
                    ),
                    format!(
                        "Search a collection of `{}` values, or unwrap with `{}(Value)`.",
                        value.name, value.underlying
                    ),
                    span,
                );
                Ty::Error
            }
            _ => self.reject_distinct_operator("in", false, left, right, span),
        }
    }

    /// Reports the operator for the first distinct operand.
    fn reject_distinct_operator(
        &mut self,
        symbol: &str,
        arithmetic: bool,
        left: &Ty,
        right: &Ty,
        span: Span,
    ) -> Ty {
        if let Ty::Distinct(distinct) = left {
            self.report_distinct_operator(symbol, arithmetic, distinct, span);
        } else if let Ty::Distinct(distinct) = right {
            self.report_distinct_operator(symbol, arithmetic, distinct, span);
        }
        Ty::Error
    }

    /// Distinct types inherit no arithmetic, logical, or string operators.
    pub(super) fn report_distinct_operator(
        &mut self,
        symbol: &str,
        arithmetic: bool,
        distinct: &DistinctTy,
        span: Span,
    ) {
        let underlying = &distinct.underlying;
        let hint = if arithmetic {
            format!(
                "Distinct types do not inherit arithmetic. Unwrap explicitly, for example `{underlying}(Value)`, or use a record or functions for quantities that need arithmetic."
            )
        } else {
            let example = if symbol == "not" {
                format!("not {underlying}(Value)")
            } else {
                format!("{underlying}(Left) {symbol} {underlying}(Right)")
            };
            format!(
                "Distinct types do not inherit this operator. Unwrap explicitly, for example `{example}`."
            )
        };
        self.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!(
                "Operator `{symbol}` is not defined for distinct type `{}`",
                distinct.name
            ),
            hint,
            span,
        );
    }
}
