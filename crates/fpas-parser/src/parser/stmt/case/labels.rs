//! Scalar labels, ranges, and existing destructuring patterns.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/README.md`.

use super::Parser;
use crate::ast::*;
use fpas_lexer::Token;

impl Parser {
    /// Parses labels that share a case arm.
    pub(super) fn parse_case_label_list(&mut self) -> Vec<CaseLabel> {
        let mut labels = Vec::new();
        labels.push(self.parse_case_label());
        while self.eat(&Token::Comma) {
            labels.push(self.parse_case_label());
        }
        labels
    }

    fn parse_case_label(&mut self) -> CaseLabel {
        let start = self.current_span();

        match self.current_token() {
            Token::Ok | Token::Error | Token::Some | Token::None => {
                let variant = match self.current_token() {
                    Token::Ok => DestructureVariant::Ok,
                    Token::Error => DestructureVariant::Error,
                    Token::Some => DestructureVariant::Some,
                    Token::None => DestructureVariant::None,
                    _ => unreachable!(),
                };
                self.advance();
                let binding = if variant == DestructureVariant::None {
                    None
                } else {
                    self.expect(&Token::LParen);
                    let binding = self.expect_ident().map(|(name, _)| name);
                    self.expect(&Token::RParen);
                    binding
                };
                return CaseLabel::Destructure {
                    variant,
                    binding,
                    span: self.span_from(start),
                };
            }
            _ => {}
        }

        let start_expr = self.parse_expression();
        let end_expr = if self.eat(&Token::DotDot) {
            Some(self.parse_expression())
        } else {
            None
        };
        CaseLabel::Value {
            start: start_expr,
            end: end_expr,
            span: self.span_from(start),
        }
    }
}
