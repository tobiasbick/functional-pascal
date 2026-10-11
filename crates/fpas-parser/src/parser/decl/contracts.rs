//! Parsing named routine entry and return clauses after the heading.
//!
//! **Documentation:** `docs/pascal/language/functions/contracts.md`.

use super::super::Parser;
use crate::ast::{EnsuresClause, RequiresClause};
use fpas_lexer::Token;

impl Parser {
    /// Parses `requires Expression;` clauses in source order.
    pub(super) fn parse_requires_clauses(&mut self) -> Vec<RequiresClause> {
        let mut clauses = Vec::new();
        while self.check(&Token::Requires) {
            let start = self.advance().span;
            let condition = self.parse_expression();
            self.expect_semi();
            let span = self.span_from(start);
            let text = span.text(&self.source).unwrap_or_default().to_owned();
            clauses.push(RequiresClause {
                condition,
                text,
                span,
            });
        }
        clauses
    }

    /// Parses `ensures [(Name)] Expression;` clauses in source order.
    pub(super) fn parse_ensures_clauses(&mut self) -> Vec<EnsuresClause> {
        let mut clauses = Vec::new();
        while self.check(&Token::Ensures) {
            let start = self.advance().span;
            let result_name = if self.check(&Token::LParen)
                && matches!(self.peek_token(), Token::Ident(_))
                && self
                    .tokens
                    .get(self.pos + 2)
                    .is_some_and(|token| token.token == Token::RParen)
            {
                self.advance();
                let name = self.expect_ident();
                self.expect(&Token::RParen);
                name
            } else {
                None
            };
            let condition = self.parse_expression();
            self.expect_semi();
            let span = self.span_from(start);
            let text = span.text(&self.source).unwrap_or_default().to_owned();
            clauses.push(EnsuresClause {
                result_name,
                condition,
                text,
                span,
            });
        }
        clauses
    }
}
