//! Logical chains and negation below comparisons.
//!
//! Documentation: `docs/pascal/language/basics/operators.md#operator-precedence`.

use super::super::{Parser, token_display};
use crate::ast::{BinaryOp, Expr, UnaryOp};
use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;
use fpas_lexer::Token;

impl Parser {
    /// Parses same-operator chains; parentheses delimit any mixed logical groups.
    pub(super) fn parse_logical(&mut self) -> Expr {
        let start = self.current_span();
        let mut left = self.parse_logical_not();
        let mut first = None;
        let mut reported_mix = false;
        loop {
            let op = match self.current_token() {
                Token::And => BinaryOp::And,
                Token::Or => BinaryOp::Or,
                Token::Xor => BinaryOp::Xor,
                _ => break,
            };
            let spelling = token_display(self.current_token());
            let first_spelling = first.get_or_insert(spelling.clone());
            if spelling != *first_spelling && !reported_mix {
                self.error_with_code(
                    PARSE_EXPECTED_EXPRESSION,
                    "Different logical operators require parentheses",
                    &format!(
                        "Choose the grouping explicitly, for example `(A {first_spelling} B) {spelling} C` or `A {first_spelling} (B {spelling} C)`."
                    ),
                    self.current_span(),
                );
                reported_mix = true;
            }
            self.advance();
            let right = self.parse_logical_not();
            left = Expr::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
                span: self.span_from(start),
            };
        }
        left
    }

    fn parse_logical_not(&mut self) -> Expr {
        if !self.check(&Token::Not) {
            return self.parse_comparison();
        }
        let start = self.current_span();
        self.advance();
        let operand = self.with_nesting(Self::parse_logical_not);
        Expr::UnaryOp {
            op: UnaryOp::Not,
            operand: Box::new(operand),
            span: self.span_from(start),
        }
    }
}
