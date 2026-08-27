//! Parser primitives (token helpers, identifier paths).
//!
//! **Documentation:** `docs/pascal/getting-started/keywords.md` (keywords), `docs/pascal/std/README.md` (`Std.*` paths).

use super::{ERROR_IDENT, Parser};
use crate::error::parse_error;
use fpas_diagnostics::codes::{PARSE_EXPECTED_IDENTIFIER, PARSE_EXPECTED_TOKEN};
use fpas_lexer::{SourcePosition, Span, SpannedToken, Token};

impl Parser {
    /// Builds a parser from a pre-lexed token stream.
    ///
    /// Always ends the stream with [`Token::Eof`]. Empty input gets a synthetic `Eof`; a truncated
    /// non-empty stream without a trailing `Eof` gets one appended so recovery loops cannot hang.
    pub fn new(tokens: Vec<SpannedToken>) -> Self {
        Self {
            tokens: ensure_trailing_eof(tokens),
            pos: 0,
            errors: Vec::new(),
            nesting_depth: 0,
            nesting_limit_reached: false,
        }
    }

    pub(crate) fn at_end(&self) -> bool {
        matches!(self.current_token(), Token::Eof)
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
        let idx = self.pos.min(self.tokens.len() - 1);
        let tok = &self.tokens[idx];
        // Never walk past the final token (always `Eof` after [`ensure_trailing_eof`]).
        if self.pos + 1 < self.tokens.len() {
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

    /// Returns true when the current token has the same **kind** as `expected`.
    ///
    /// Only pass unit-variant tokens (no payload), e.g. [`Token::Semicolon`]. Payload-bearing
    /// variants such as [`Token::Ident`] or [`Token::Integer`] must use explicit matching.
    pub(crate) fn check(&self, expected: &Token) -> bool {
        std::mem::discriminant(self.current_token()) == std::mem::discriminant(expected)
    }

    pub(crate) fn is_comparison_token(&self) -> bool {
        matches!(
            self.current_token(),
            Token::Equal
                | Token::NotEqual
                | Token::Less
                | Token::Greater
                | Token::LessEqual
                | Token::GreaterEqual
                | Token::In
        )
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
        if self.nesting_limit_reached {
            return;
        }
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

    pub(crate) fn error_ident(&self, span: Span) -> (String, Span) {
        (ERROR_IDENT.to_owned(), span)
    }

    pub(crate) fn expect_ident_or_error(&mut self, start: Span) -> (String, Span) {
        match self.expect_ident() {
            Some(ident) => ident,
            None => {
                if !self.at_end() {
                    self.advance();
                }
                self.error_ident(start)
            }
        }
    }

    pub(crate) fn is_mutable_var_start(&self) -> bool {
        matches!(self.current_token(), Token::Mutable) && self.peek_token() == &Token::Var
    }

    pub(crate) fn expect_ident(&mut self) -> Option<(String, Span)> {
        if let Some(ident) = self.try_consume_soft_keyword_ident() {
            return Some(ident);
        }
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

    /// Reserved words that name `Std.*` units (`array`, `result`, `option`, `dict`) are lexed as
    /// keywords. When they appear as a dotted path segment, treat them like the corresponding
    /// Pascal-cased identifier (`Array`, …).
    ///
    /// The first segment of a path uses the same rule in [`Parser::parse_qualified_id`] and
    /// [`Parser::parse_designator`]. In expression position, those keywords only start a
    /// designator when immediately followed by `.` (so `array of T` remains the array type and
    /// `result of T, E` remains the result type).
    ///
    /// `event` / `property` are also soft-accepted as identifiers outside record-member
    /// declaration position so existing `Std.*.Event` type names keep working.
    pub(crate) fn try_consume_std_keyword_path_segment(&mut self) -> Option<(String, Span)> {
        let segment = match self.current_token() {
            Token::Array => "Array",
            Token::Result => "Result",
            Token::OptionKw => "Option",
            Token::Dict => "Dict",
            Token::Event => "Event",
            Token::Property => "Property",
            _ => return None,
        };
        let span = self.current_span();
        self.advance();
        Some((segment.to_owned(), span))
    }

    /// Soft-accept selected keywords as identifiers (preserving Std type names).
    pub(crate) fn try_consume_soft_keyword_ident(&mut self) -> Option<(String, Span)> {
        let name = match self.current_token() {
            Token::Event => "Event",
            Token::Property => "Property",
            _ => return None,
        };
        let span = self.current_span();
        self.advance();
        Some((name.to_owned(), span))
    }

    /// True when the current token can start an identifier designator, including soft
    /// keywords such as `Event` / `Property` used as names.
    pub(crate) fn is_ident_designator_start(&self) -> bool {
        matches!(
            self.current_token(),
            Token::Ident(_) | Token::Event | Token::Property
        ) || self.is_std_keyword_path_start()
    }

    pub(crate) fn is_std_keyword_path_start(&self) -> bool {
        matches!(
            self.current_token(),
            Token::Array | Token::Result | Token::OptionKw | Token::Dict
        ) && matches!(self.peek_token(), Token::Dot)
    }

    /// Identifier segment after `.` (including `array` → `Array`, etc.).
    pub(crate) fn expect_ident_after_dot(&mut self) -> Option<(String, Span)> {
        if let Some(seg) = self.try_consume_std_keyword_path_segment() {
            return Some(seg);
        }
        self.expect_ident()
    }

    /// Returns whether the current token belongs to the parent construct of a missing expression.
    pub(crate) fn is_expression_recovery_boundary(&self) -> bool {
        matches!(
            self.current_token(),
            Token::End
                | Token::Then
                | Token::Do
                | Token::Until
                | Token::Else
                | Token::Of
                | Token::To
                | Token::Downto
                | Token::In
                | Token::RParen
                | Token::RBracket
                | Token::Comma
                | Token::Colon
                | Token::Semicolon
                | Token::Dot
                | Token::Eof
        )
    }

    pub(crate) fn expect_semi(&mut self) {
        self.expect(&Token::Semicolon);
    }
}

fn ensure_trailing_eof(mut tokens: Vec<SpannedToken>) -> Vec<SpannedToken> {
    if tokens.is_empty() {
        tokens.push(SpannedToken {
            token: Token::Eof,
            span: Span {
                offset: 0,
                length: 0,
                line: 1,
                column: 1,
                source_id: 0,
            },
            end: SourcePosition {
                offset: 0,
                line: 1,
                column: 1,
            },
        });
        return tokens;
    }
    if !matches!(tokens.last().map(|t| &t.token), Some(Token::Eof)) {
        let last = &tokens[tokens.len() - 1];
        let last_span = last.span;
        let span = Span {
            offset: last.end.offset,
            length: 0,
            line: last.end.line,
            column: last.end.column,
            source_id: last_span.source_id,
        };
        tokens.push(SpannedToken {
            token: Token::Eof,
            span,
            end: last.end,
        });
    }
    tokens
}
