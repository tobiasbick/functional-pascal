//! `if C then A elsif D then B else E end if` in expression positions.
//!
//! A statement that starts with `if` is always the `if` statement; this form is
//! reached only where an expression is expected.
//!
//! **Documentation:** `docs/pascal/language/control-flow/if-then-else.md`

use super::super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::{
    PARSE_IF_EXPRESSION_WITHOUT_ELSE, PARSE_STATEMENT_IN_EXPRESSION_BRANCH,
};
use fpas_lexer::Token;

impl Parser {
    /// Parses an `if` expression whose `end if` closes it.
    pub(super) fn parse_if_expression(&mut self) -> Expr {
        self.with_block_closer(Token::If, Self::parse_if_expression_inner)
    }

    fn parse_if_expression_inner(&mut self) -> Expr {
        let start = self.advance().span;
        let mut branches = vec![self.parse_if_expression_branch(start)];
        while self.check(&Token::Elsif) {
            let keyword = self.advance().span;
            branches.push(self.parse_if_expression_branch(keyword));
        }
        let else_start = self.current_span();
        let else_value = if self.eat(&Token::Else) {
            let value = self.parse_expression();
            self.reject_statement_in_branch();
            value
        } else {
            let span = self.current_span();
            self.error_with_code(
                PARSE_IF_EXPRESSION_WITHOUT_ELSE,
                "An `if` expression requires an `else` branch",
                "Add `else Value` before `end if`, for example `if Ready then 1 else 0 end if`. Use an `if` statement to run statements without producing a value.",
                span,
            );
            Expr::Error(span)
        };
        let else_span = self.span_from(else_start);
        self.expect_expression_end(&Token::If);
        Expr::If {
            branches,
            else_value: Box::new(else_value),
            else_span,
            span: self.span_from(start),
        }
    }

    /// Parses `Condition then Value` after its `if` or `elsif` keyword at `keyword`.
    fn parse_if_expression_branch(&mut self, keyword: fpas_lexer::Span) -> IfExprBranch {
        let condition = self.parse_expression();
        self.expect(&Token::Then);
        let value = self.parse_expression();
        let span = self.span_from(keyword);
        self.reject_statement_in_branch();
        IfExprBranch {
            condition,
            value,
            span,
        }
    }

    /// Reports `:=`, or `;` before `elsif`, `else`, or `end if`, after a branch value
    /// and skips to the next branch keyword.
    ///
    /// A `;` followed by anything else ends the enclosing statement, so the missing
    /// `else` or `end if` is reported instead.
    fn reject_statement_in_branch(&mut self) {
        let statement = match self.current_token() {
            Token::ColonAssign => true,
            Token::Semicolon => match self.peek_token() {
                Token::Elsif | Token::Else => true,
                Token::End => {
                    self.tokens.get(self.pos + 2).map(|token| &token.token) == Some(&Token::If)
                }
                _ => false,
            },
            _ => false,
        };
        if !statement {
            return;
        }
        self.error_with_code(
            PARSE_STATEMENT_IN_EXPRESSION_BRANCH,
            "An `if` expression branch must be a single expression",
            "Write one value per branch without `;`, for example `if Ready then 1 else 0 end if`. Use an `if` statement for assignments or several statements.",
            self.current_span(),
        );
        while !matches!(
            self.current_token(),
            Token::Elsif | Token::Else | Token::End | Token::Eof
        ) {
            self.advance();
        }
    }
}
