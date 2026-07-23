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
        let mut raw = String::new();

        while !self.at_end() {
            let ch = self.current();
            if !ch.is_ascii_alphanumeric() && ch != b'_' {
                break;
            }
            raw.push(self.advance() as char);
        }

        // Non-ASCII letters/digits after an ASCII prefix (e.g. `café`) are not
        // valid identifier characters — consume the run and emit one diagnostic
        // instead of splitting into `Ident("caf")` + unexpected `é`.
        if self.has_non_ascii_ident_continuation() {
            while self.has_non_ascii_ident_continuation() {
                self.advance_utf8_char();
            }
            self.push_err(
                LEX_NON_ASCII_IN_IDENTIFIER,
                "Identifiers may only use ASCII letters, digits, and `_`",
                "Replace non-ASCII characters with ASCII, for example `cafe` instead of `café`.",
                so,
                sl,
                sc,
            );
            return;
        }

        let token = Token::from_ident_owned(raw);
        self.push_tok(token, so, sl, sc);
    }

    fn has_non_ascii_ident_continuation(&self) -> bool {
        self.remaining_str()
            .chars()
            .next()
            .is_some_and(|ch| !ch.is_ascii() && (ch.is_alphanumeric() || ch == '_'))
    }
}
