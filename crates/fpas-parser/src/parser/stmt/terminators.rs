//! Statement terminators and recovery at statement-list boundaries.
//!
//! **Documentation:** `docs/pascal/tools/fmt-style.md#semicolons`

use super::super::Parser;
use crate::ast::Stmt;
use crate::error::parse_error;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::{Span, Token};

impl Parser {
    /// Parses statements with a required terminator, preserving boundary tokens during recovery.
    pub(crate) fn parse_statement_list(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while !self.is_stmt_list_end() {
            statements.push(self.parse_terminated_statement());
        }
        statements
    }

    /// Parses one terminated statement, including the terminator after each named ending.
    pub(super) fn parse_terminated_statement(&mut self) -> Stmt {
        let errors_before = self.errors.len();
        let statement = self.parse_statement();
        if self.errors.len() > errors_before
            && matches!(
                statement,
                Stmt::If { .. }
                    | Stmt::Case { .. }
                    | Stmt::For { .. }
                    | Stmt::ForIn { .. }
                    | Stmt::While { .. }
            )
            && self.is_stmt_list_end()
        {
            return statement;
        }
        self.expect_statement_terminator();
        statement
    }

    fn expect_statement_terminator(&mut self) {
        if self.eat(&Token::Semicolon) {
            return;
        }
        let found = crate::parser::token_display(self.current_token());
        let error = parse_error(
            PARSE_EXPECTED_TOKEN,
            format!("Expected `;` after statement, found `{found}`"),
            "Terminate the preceding statement with `;`, for example `WriteLn('value');`.",
            self.current_span(),
        )
        .with_expected_found(";", found);
        if !self.nesting_limit_reached {
            self.errors.push(error);
        }
        while !self.is_stmt_list_end()
            && !self.check(&Token::Semicolon)
            && !self.can_start_statement()
        {
            self.advance();
        }
        self.eat(&Token::Semicolon);
    }

    /// Recognizes enclosing boundaries without consuming them during recovery.
    pub(super) fn is_stmt_list_end(&self) -> bool {
        matches!(
            self.current_token(),
            Token::End
                | Token::Else
                | Token::Until
                | Token::Eof
                | Token::Program
                | Token::Unit
                | Token::Type
                | Token::Const
                | Token::Public
                | Token::Static
        ) || self.check(&Token::Elsif)
            && self.peek_token() != &Token::ColonAssign
            && self.block_closers.contains(&Token::If)
            || self.check(&Token::When)
                && self.peek_token() != &Token::ColonAssign
                && self.block_closers.contains(&Token::Case)
            || matches!(self.current_token(), Token::Function | Token::Procedure)
                && matches!(self.peek_token(), Token::Ident(_))
    }

    /// Keeps the last case-arm terminator outside the arm's source span.
    pub(super) fn span_before_terminator(&self, start: Span) -> Span {
        let mut span = self.span_from(start);
        if self.pos > 0
            && self.tokens[self.pos - 1].token == Token::Semicolon
            && let Some(previous) = self
                .pos
                .checked_sub(2)
                .and_then(|index| self.tokens.get(index))
        {
            span.length =
                (previous.span.offset + previous.span.length).saturating_sub(start.offset);
        }
        span
    }
}
