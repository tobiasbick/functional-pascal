//! Recursive patterns in case labels, payload positions, and `is` tests.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/syntax.md`.

use super::Parser;
use crate::ast::*;
use fpas_lexer::Token;

impl Parser {
    /// True when the label starts with `Name(` or `Type.Name(`.
    pub(in crate::parser) fn at_variant_pattern(&self) -> bool {
        let mut position = self.pos;
        loop {
            if !matches!(self.token_at(position), Some(Token::Ident(_))) {
                return false;
            }
            position += 1;
            match self.token_at(position) {
                Some(Token::Dot) => position += 1,
                Some(Token::LParen) => return true,
                _ => return false,
            }
        }
    }

    /// Parses one pattern: `const Name`, `_`, a variant or Result/Option pattern, or a value.
    pub(in crate::parser) fn parse_pattern(&mut self) -> Pattern {
        let start = self.current_span();
        match self.current_token() {
            Token::Const => {
                self.advance();
                let name = self
                    .expect_ident()
                    .map_or_else(|| crate::parser::ERROR_IDENT.to_string(), |(name, _)| name);
                Pattern::Binding {
                    name,
                    span: self.span_from(start),
                }
            }
            Token::Ident(name)
                if name == "_" && matches!(self.peek_token(), Token::Comma | Token::RParen) =>
            {
                self.advance();
                Pattern::Wildcard(start)
            }
            Token::Ok | Token::Error | Token::Some | Token::None => {
                let variant = match self.current_token() {
                    Token::Ok => DestructureVariant::Ok,
                    Token::Error => DestructureVariant::Error,
                    Token::Some => DestructureVariant::Some,
                    _ => DestructureVariant::None,
                };
                self.advance();
                let payload = if variant == DestructureVariant::None {
                    None
                } else {
                    self.expect(&Token::LParen);
                    let payload = self.with_nesting(Self::parse_pattern);
                    self.expect(&Token::RParen);
                    Some(Box::new(payload))
                };
                Pattern::Destructure {
                    variant,
                    payload,
                    span: self.span_from(start),
                }
            }
            _ if self.at_variant_pattern() => {
                let constructor = self.parse_designator();
                let fields = self.parse_pattern_fields();
                Pattern::Variant {
                    constructor,
                    fields,
                    span: self.span_from(start),
                }
            }
            // Comparison values bind tighter than `and`, so `X is 0 and Ready` stays a conjunction.
            _ => Pattern::Value(self.parse_additive()),
        }
    }

    /// Parses `(Pattern, ...)` after a variant name; an empty list is allowed.
    fn parse_pattern_fields(&mut self) -> Vec<PatternField> {
        self.expect(&Token::LParen);
        let mut fields = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                let label = match (self.current_token(), self.peek_token()) {
                    (Token::Ident(name), Token::ColonAssign) => {
                        let label = (name.clone(), self.current_span());
                        self.advance();
                        self.advance();
                        Some(label)
                    }
                    _ => None,
                };
                let pattern = self.with_nesting(Self::parse_pattern);
                fields.push(PatternField { label, pattern });
                if !self.eat(&Token::Comma) {
                    break;
                }
            }
        }
        self.expect(&Token::RParen);
        fields
    }

    /// True when a `Name(...)` label is followed by `..`, so it starts a value range
    /// such as `UserId(1)..UserId(9)`.
    ///
    /// **Documentation:** `docs/pascal/language/types/distinct-types.md`
    pub(in crate::parser) fn at_call_range_start(&self) -> bool {
        let Some(mut position) = (self.pos..self.tokens.len())
            .find(|&position| self.token_at(position) == Some(&Token::LParen))
        else {
            return false;
        };
        let mut depth = 0usize;
        while let Some(token) = self.token_at(position) {
            match token {
                Token::LParen => depth += 1,
                Token::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        return self.token_at(position + 1) == Some(&Token::DotDot);
                    }
                }
                Token::Eof => return false,
                _ => {}
            }
            position += 1;
        }
        false
    }

    fn token_at(&self, position: usize) -> Option<&Token> {
        self.tokens.get(position).map(|token| &token.token)
    }
}
