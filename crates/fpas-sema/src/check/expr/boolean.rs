//! Boolean-only logical typing and actionable integer-bit migration hints.
//!
//! Documentation: `docs/pascal/language/basics/operators.md`.

use super::super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::BinaryOp;

impl Checker {
    /// Checks Boolean negation, suggesting `Std.Bits.BitNot` for an integer operand.
    pub(super) fn check_boolean_not(&mut self, operand: &Ty, span: Span) -> Ty {
        let operand = self.resolve_visible_type(operand);
        if operand.is_error() {
            return Ty::Error;
        }
        if operand == Ty::Boolean {
            return Ty::Boolean;
        }
        let hint = if operand == Ty::Integer {
            "For integer bits, use `Std.Bits.BitNot(Value)` and import `uses Std.Bits;`."
        } else {
            "Use a boolean operand, such as `not (Count > 0)`."
        };
        self.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("`not` requires a boolean operand, got {operand}"),
            hint,
            span,
        );
        Ty::Error
    }

    /// Checks logical operands without implicit conversion between integers and booleans.
    pub(super) fn check_boolean_binary(
        &mut self,
        op: BinaryOp,
        left: &Ty,
        right: &Ty,
        span: Span,
    ) -> Ty {
        if *left == Ty::Boolean && *right == Ty::Boolean {
            return Ty::Boolean;
        }
        let (operator, replacement) = match op {
            BinaryOp::And => ("and", "BitAnd"),
            BinaryOp::Or => ("or", "BitOr"),
            BinaryOp::Xor => ("xor", "BitXor"),
            _ => unreachable!("logical operator checked by caller"),
        };
        let hint = if *left == Ty::Integer && *right == Ty::Integer {
            format!(
                "For integer bits, use `Std.Bits.{replacement}(Left, Right)` and import `uses Std.Bits;`."
            )
        } else {
            "Both operands must be boolean; compare numeric values explicitly, for example `Count > 0`.".into()
        };
        self.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("Operator `{operator}` requires boolean operands, got {left} and {right}"),
            hint,
            span,
        );
        Ty::Error
    }
}
