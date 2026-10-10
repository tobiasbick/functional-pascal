//! `case Value of when Labels: Expression; ... [else Expression;] end case` in
//! expression positions.
//!
//! A statement that starts with `case` is always the `case` statement; this form
//! is reached only where an expression is expected.
//!
//! **Documentation:** `docs/pascal/language/control-flow/case-of-intro.md`

use super::super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::{PARSE_EXPECTED_TOKEN, PARSE_STATEMENT_IN_EXPRESSION_BRANCH};
use fpas_lexer::Token;

impl Parser {
    /// Parses a `case` expression whose `end case` closes it.
    pub(super) fn parse_case_expression(&mut self) -> Expr {
        self.with_block_closer(Token::Case, Self::parse_case_expression_inner)
    }

    fn parse_case_expression_inner(&mut self) -> Expr {
        let start = self.advance().span;
        let selector = self.parse_expression();
        self.expect(&Token::Of);
        if !self.check(&Token::When) {
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "Expected at least one case arm",
                "Add an arm such as `when 1: 'one';` after `of`.",
                self.current_span(),
            );
        }
        let mut arms = Vec::new();
        while self.check(&Token::When) {
            arms.push(self.parse_case_expression_arm());
        }
        let else_arm = if self.check(&Token::Else) {
            let else_start = self.advance().span;
            let value = self.parse_expression();
            let span = self.span_from(else_start);
            self.expect_case_arm_end();
            Some(Box::new(CaseExprElse { value, span }))
        } else {
            None
        };
        self.expect_expression_end(&Token::Case);
        Expr::Case {
            selector: Box::new(selector),
            arms,
            else_arm,
            span: self.span_from(start),
        }
    }

    fn parse_case_expression_arm(&mut self) -> CaseExprArm {
        let start = self.advance().span;
        let labels = self.parse_case_label_list();
        let guard = if self.eat(&Token::If) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect(&Token::Colon);
        let value = self.parse_expression();
        let span = self.span_from(start);
        self.expect_case_arm_end();
        CaseExprArm {
            labels,
            guard,
            value,
            span,
        }
    }

    /// Requires the `;` after an arm value and rejects statements inside the arm.
    fn expect_case_arm_end(&mut self) {
        let statement = match self.current_token() {
            Token::ColonAssign => true,
            Token::Semicolon => !matches!(
                self.peek_token(),
                Token::When | Token::Else | Token::End | Token::Eof
            ),
            _ => {
                self.expect(&Token::Semicolon);
                return;
            }
        };
        if !statement {
            self.advance();
            return;
        }
        self.error_with_code(
            PARSE_STATEMENT_IN_EXPRESSION_BRANCH,
            "A `case` expression arm must be a single expression",
            "Write one value per arm followed by `;`, for example `when 1: 'one';`. Use a `case` statement for assignments or several statements.",
            self.current_span(),
        );
        while !matches!(
            self.current_token(),
            Token::When | Token::Else | Token::End | Token::Eof
        ) {
            self.advance();
        }
    }
}
