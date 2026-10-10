//! Scalar labels, ranges, explicit bindings, and pattern labels.
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
            Token::Const => {
                self.advance();
                let name = self
                    .expect_ident()
                    .map_or_else(|| crate::parser::ERROR_IDENT.to_string(), |(name, _)| name);
                return CaseLabel::Binding {
                    name,
                    span: self.span_from(start),
                };
            }
            Token::Ok | Token::Error | Token::Some | Token::None => {
                return CaseLabel::Pattern(self.parse_pattern());
            }
            _ if self.at_variant_pattern() && !self.at_call_range_start() => {
                return CaseLabel::Pattern(self.parse_pattern());
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
