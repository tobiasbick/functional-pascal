//! Explicit case-arm boundaries and scoped statement-list bodies.
//!
//! **Documentation:** `docs/pascal/language/control-flow/case-of-intro.md`.

mod labels;

use super::super::Parser;
use crate::ast::*;
use crate::error::parse_error;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    /// Parses `when` arms and an optional catch-all before the matching `end case`.
    pub(super) fn parse_case_stmt(&mut self) -> Stmt {
        self.with_block_closer(Token::Case, Self::parse_case_stmt_inner)
    }

    fn parse_case_stmt_inner(&mut self) -> Stmt {
        let start = self.advance().span;
        let expr = self.parse_expression();
        self.expect(&Token::Of);

        let mut arms = Vec::new();
        if matches!(self.current_token(), Token::Else | Token::End | Token::Eof) {
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "Expected at least one case arm",
                "Add an arm such as `when 1: null;` after `of`.",
                self.current_span(),
            );
        }
        while self.check(&Token::When) || !self.is_stmt_list_end() {
            if self.check(&Token::When) {
                arms.push(self.parse_case_arm());
                continue;
            }
            let found = super::super::token_display(self.current_token());
            self.errors.push(
                parse_error(
                    PARSE_EXPECTED_TOKEN,
                    format!("Expected `when` at the start of a case arm, found `{found}`"),
                    "Start each arm with `when`, for example `when 1: Value := 1;`.",
                    self.current_span(),
                )
                .with_expected_found("when", found),
            );
            while !self.check(&Token::When) && !self.is_stmt_list_end() {
                self.advance();
            }
        }

        let else_body = if self.eat(&Token::Else) {
            let statements = self.parse_statement_list();
            self.require_control_statements(&statements);
            Some(statements)
        } else {
            None
        };
        self.expect_block_end(&Token::Case);
        Stmt::Case {
            expr,
            arms,
            else_body,
            span: self.span_from(start),
        }
    }

    fn parse_case_arm(&mut self) -> CaseArm {
        let start = self.advance().span;
        let labels = self.parse_case_label_list();
        let guard = if self.eat(&Token::If) {
            Some(self.parse_expression())
        } else {
            None
        };
        let body_start = self.expect(&Token::Colon).unwrap_or(self.current_span());
        let body = self.parse_control_body(body_start);
        CaseArm {
            labels,
            guard,
            body,
            span: self.span_before_terminator(start),
        }
    }
}
