//! Parse value branches without statement lists or implicit final values.

use super::super::Parser;
use crate::ast::{CaseArm, CaseExpr, Expr, IfExpr};
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    /// Parse a conditional value with one expression per branch and a required fallback.
    pub(super) fn parse_if_expression(&mut self) -> Expr {
        let start = self.advance().span;
        let condition = self.parse_expression();
        self.expect(&Token::Then);
        let then_value = self.parse_expression();
        let mut elsif_values = Vec::new();
        while self.eat(&Token::Elsif) {
            let condition = self.parse_expression();
            self.expect(&Token::Then);
            elsif_values.push((condition, self.parse_expression()));
        }
        let else_value = if self.eat(&Token::Else) {
            self.parse_expression()
        } else {
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "An if expression requires else",
                "Write `if Condition then Value else OtherValue end if`.",
                self.current_span(),
            );
            Expr::Error(self.current_span())
        };
        self.expect_named_end(&Token::If);
        Expr::If(Box::new(IfExpr {
            condition,
            then_value,
            elsif_values,
            else_value,
            span: self.span_from(start),
        }))
    }

    /// Parse pattern-selected values with arm terminators owned by the case expression.
    pub(super) fn parse_case_expression(&mut self) -> Expr {
        let start = self.advance().span;
        let value = self.parse_expression();
        self.expect(&Token::Of);
        let mut arms = Vec::new();
        while self.eat(&Token::When) {
            let arm_start = self.tokens[self.pos - 1].span;
            let labels = self.parse_case_label_list();
            let guard = self.eat(&Token::If).then(|| self.parse_expression());
            self.expect(&Token::Colon);
            let body = self.parse_expression();
            self.expect_semi();
            arms.push(CaseArm {
                labels,
                guard,
                body,
                span: self.span_from(arm_start),
            });
        }
        if arms.is_empty() {
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "A case expression requires at least one when arm",
                "Write `case Value of when Pattern: Expression; ... end case`.",
                self.current_span(),
            );
        }
        let else_value = if self.eat(&Token::Else) {
            let value = self.parse_expression();
            self.expect_semi();
            Some(value)
        } else {
            None
        };
        self.expect_named_end(&Token::Case);
        Expr::Case(Box::new(CaseExpr {
            value,
            arms,
            else_value,
            span: self.span_from(start),
        }))
    }
}
