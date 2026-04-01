use super::Parser;
use crate::error::parse_error;
use fpas_diagnostics::codes::{PARSE_EXPECTED_IDENTIFIER, PARSE_EXPECTED_TOKEN};
use fpas_lexer::{Span, SpannedToken, Token};

impl Parser {
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    pub(crate) fn at_end(&self) -> bool {
        self.current_token() == &Token::Eof
    }

    pub(crate) fn current(&self) -> &SpannedToken {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    pub(crate) fn current_token(&self) -> &Token {
        &self.current().token
    }

    pub(crate) fn current_span(&self) -> Span {
        self.current().span
    }

    pub(crate) fn peek_token(&self) -> &Token {
        let idx = (self.pos + 1).min(self.tokens.len() - 1);
        &self.tokens[idx].token
    }

    pub(crate) fn advance(&mut self) -> &SpannedToken {
        let tok = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if !self.at_end() {
            self.pos += 1;
        }
        tok
    }

    pub(crate) fn expect(&mut self, expected: &Token) -> Option<Span> {
        if self.check(expected) {
            Some(self.advance().span)
        } else {
            let span = self.current_span();
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                &format!(
                    "Expected `{expected_str}`, found `{found}`",
                    expected_str = super::token_display(expected),
                    found = super::token_display(self.current_token()),
                ),
                &format!("Insert `{}` here.", super::token_display(expected)),
                span,
            );
            None
        }
    }

    pub(crate) fn check(&self, expected: &Token) -> bool {
        std::mem::discriminant(self.current_token()) == std::mem::discriminant(expected)
    }

    pub(crate) fn eat(&mut self, expected: &Token) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub(crate) fn error_with_code(
        &mut self,
        code: fpas_diagnostics::DiagnosticCode,
        message: &str,
        hint: &str,
        span: Span,
    ) {
        self.errors.push(parse_error(code, message, hint, span));
    }

    pub(crate) fn span_from(&self, start: Span) -> Span {
        if self.pos == 0 {
            return start;
        }
        let end = &self.tokens[(self.pos - 1).min(self.tokens.len() - 1)];
        Span {
            offset: start.offset,
            length: (end.span.offset + end.span.length).saturating_sub(start.offset),
            line: start.line,
            column: start.column,
            source_id: start.source_id,
        }
    }

    pub(crate) fn expect_ident(&mut self) -> Option<(String, Span)> {
        let Token::Ident(name) = self.current_token().clone() else {
            let span = self.current_span();
            self.error_with_code(
                PARSE_EXPECTED_IDENTIFIER,
                &format!(
                    "Expected identifier, found `{}`",
                    super::token_display(self.current_token())
                ),
                "An identifier (name) is required here.",
                span,
            );
            return None;
        };
        let span = self.advance().span;
        Some((name, span))
    }

    /// Identifier segment after `.` (allows `array`, `result`, `option` as names
    /// so `Std.Array`, `Std.Result`, `Std.Option` work).
    pub(crate) fn expect_ident_after_dot(&mut self) -> Option<(String, Span)> {
        let span = self.current_span();
        match self.current_token() {
            Token::Ident(_) => self.expect_ident(),
            Token::Array => {
                self.advance();
                Some(("Array".to_string(), span))
            }
            Token::Result => {
                self.advance();
                Some(("Result".to_string(), span))
            }
            Token::OptionKw => {
                self.advance();
                Some(("Option".to_string(), span))
            }
            Token::Dict => {
                self.advance();
                Some(("Dict".to_string(), span))
            }
            _ => self.expect_ident(),
        }
    }

    pub(crate) fn expect_semi(&mut self) {
        self.expect(&Token::Semicolon);
    }
}
