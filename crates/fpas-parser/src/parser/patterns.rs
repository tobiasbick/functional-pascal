//! Recursive patterns shared by statement and value-producing cases.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/README.md`.

use super::Parser;
use crate::ast::{Designator, DesignatorPart, Pattern};
use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;
use fpas_lexer::Token;

impl Parser {
    /// Parse a recursive pattern with explicit bindings and payload wildcards.
    pub(in crate::parser) fn parse_pattern(&mut self) -> Pattern {
        self.with_nesting(Self::parse_pattern_inner)
    }

    fn parse_pattern_inner(&mut self) -> Pattern {
        let start = self.current_span();
        if self.eat(&Token::Const) {
            let (name, _) = self
                .expect_ident()
                .unwrap_or_else(|| self.error_ident(start));
            return Pattern::Binding {
                name,
                span: self.span_from(start),
            };
        }
        if matches!(self.current_token(), Token::Ident(name) if name == "_") {
            self.advance();
            return Pattern::Wildcard(start);
        }
        if self.at_pattern_variant() {
            let mut parts = Vec::new();
            if let Some(owner) = self.recover_unqualified_builtin_pattern() {
                parts.push(DesignatorPart::Ident(owner.into(), start));
            }
            loop {
                let span = self.current_span();
                let name = match self.current_token() {
                    Token::Ident(name) => name.clone(),
                    Token::OptionKw => "Option".into(),
                    Token::Result => "Result".into(),
                    Token::Some => "Some".into(),
                    Token::None => "None".into(),
                    Token::Ok => "Ok".into(),
                    Token::Error => "Error".into(),
                    _ => break,
                };
                self.advance();
                parts.push(DesignatorPart::Ident(name, span));
                if !self.eat(&Token::Dot) {
                    break;
                }
            }
            let designator = Designator {
                parts,
                span: self.span_from(start),
            };
            let mut arguments = Vec::new();
            let parenthesized = self.eat(&Token::LParen);
            if parenthesized {
                if !self.check(&Token::RParen) {
                    loop {
                        arguments.push(self.parse_pattern());
                        if !self.eat(&Token::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&Token::RParen);
            }
            return Pattern::Variant {
                designator,
                arguments,
                parenthesized,
                span: self.span_from(start),
            };
        }
        let value = self.parse_expression();
        let end = self.eat(&Token::DotDot).then(|| self.parse_expression());
        Pattern::Value {
            start: value,
            end,
            span: self.span_from(start),
        }
    }

    // Reports a bare builtin variant and returns its implied owner so payload patterns still parse.
    fn recover_unqualified_builtin_pattern(&mut self) -> Option<&'static str> {
        let owner = match self.current_token() {
            Token::Some | Token::None => "Option",
            Token::Ok | Token::Error => "Result",
            _ => return None,
        };
        self.error_with_code(
            PARSE_EXPECTED_EXPRESSION,
            "Builtin variant requires its type qualifier",
            if owner == "Option" {
                "Write `Option.Some(const Value)` or `Option.None`."
            } else {
                "Write `Result.Ok(const Value)` or `Result.Error(const Message)`."
            },
            self.current_span(),
        );
        Some(owner)
    }

    fn at_pattern_variant(&self) -> bool {
        if matches!(
            self.current_token(),
            Token::OptionKw | Token::Result | Token::Some | Token::None | Token::Ok | Token::Error
        ) {
            return true;
        }
        let mut position = self.pos;
        let mut qualified = false;
        while matches!(self.tokens[position].token, Token::Ident(_)) {
            position += 1;
            if matches!(self.tokens[position].token, Token::LParen) {
                return qualified;
            }
            if !matches!(self.tokens[position].token, Token::Dot) {
                return false;
            }
            qualified = true;
            position += 1;
        }
        false
    }
}
