//! Scans identifiers and maps them to keywords.
//!
//! Identifiers are ASCII letters, digits, and `_` only (Pascal case-insensitive keywords).
//!
//! **Documentation:** `docs/specs/grammar.ebnf` (`identifier`), `docs/pascal/getting-started/keywords.md`

use super::Lexer;
use crate::Token;
use fpas_diagnostics::codes::LEX_NON_ASCII_IN_IDENTIFIER;

impl Lexer<'_> {
    pub(super) fn scan_ident_or_keyword(&mut self) {
        let (so, sl, sc) = self.span_here();

        while !self.at_end() {
            let ch = self.current();
            if !ch.is_ascii_alphanumeric() && ch != b'_' {
                break;
            }
            self.advance();
        }

        if self.starts_non_ascii_identifier() {
            self.consume_identifier_recovery();
            self.report_invalid_identifier(so, sl, sc);
            return;
        }

        let raw = &self.source[so..self.pos];
        let token = Token::from_ident(raw);
        self.push_tok(token, so, sl, sc);
    }

    pub(super) fn starts_non_ascii_identifier(&self) -> bool {
        self.remaining_str()
            .chars()
            .next()
            .is_some_and(|ch| !ch.is_ascii() && ch.is_alphanumeric())
    }

    pub(super) fn scan_invalid_identifier(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.consume_identifier_recovery();
        self.report_invalid_identifier(so, sl, sc);
    }

    fn consume_identifier_recovery(&mut self) {
        while self
            .remaining_str()
            .chars()
            .next()
            .is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
        {
            self.advance_utf8_char();
        }
    }

    fn report_invalid_identifier(&mut self, offset: usize, line: u32, column: u32) {
        self.push_err(
            LEX_NON_ASCII_IN_IDENTIFIER,
            "Identifiers may only use ASCII letters, digits, and `_`",
            "Replace non-ASCII characters with ASCII, for example `cafe` instead of `café`.",
            offset,
            line,
            column,
        );
    }
}
