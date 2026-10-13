//! Parsing named routine entry and return clauses after the heading.
//!
//! **Documentation:** `docs/pascal/language/functions/contracts.md`.

use super::super::Parser;
use crate::ast::{EnsuresClause, Expr, RequiresClause};
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

    /// Parses `ensures [(Name):] Expression;` clauses in source order.
    pub(super) fn parse_ensures_clauses(&mut self) -> Vec<EnsuresClause> {
        let mut clauses = Vec::new();
        while self.check(&Token::Ensures) {
            let start = self.advance().span;
            let named_group = self.check(&Token::LParen)
                && matches!(self.peek_token(), Token::Ident(_))
                && self
                    .tokens
                    .get(self.pos + 2)
                    .is_some_and(|token| token.token == Token::RParen);
            let after_group = self.tokens.get(self.pos + 3).map(|token| &token.token);
            let legacy_binder = named_group
                && matches!(
                    after_group,
                    Some(
                        Token::Ident(_)
                            | Token::Integer(_)
                            | Token::Real(_)
                            | Token::Str(_)
                            | Token::True
                            | Token::False
                            | Token::Some
                            | Token::None
                            | Token::Ok
                            | Token::Error
                            | Token::Array
                            | Token::Not
                            | Token::If
                            | Token::Case
                            | Token::Try
                            | Token::Go
                            | Token::Function
                            | Token::Procedure
                            | Token::LParen
                    )
                );
            let result_name = if named_group && after_group == Some(&Token::Colon) {
                self.advance();
                let name = self.expect_ident();
                self.expect(&Token::RParen);
                self.expect(&Token::Colon);
                name
            } else {
                None
            };
            let mut condition = self.parse_expression();
            if legacy_binder {
                self.error_with_code(
                    fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN,
                    "A postcondition return binder requires `:` after `(Name)`",
                    "Write `ensures (Returned): Returned > 0;`. Without a binder, write a normal Boolean expression such as `ensures (Flag);`.",
                    self.current_span(),
                );
                // Consume the adjacent old condition so the following clause/body can recover.
                let _ = self.parse_expression();
                condition = Expr::Error(self.span_from(start));
            }
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
