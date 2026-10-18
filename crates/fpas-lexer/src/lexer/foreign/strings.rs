//! Double-quoted strings from the C family.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/basics/primitive-types.md`

use crate::Token;
use crate::lexer::Lexer;
use fpas_diagnostics::Applicability;

impl Lexer<'_> {
    /// Scans `"text"` on one line as a string literal and reports it once.
    pub(in crate::lexer) fn scan_double_quoted_string(&mut self) {
        let start = self.span_here();
        self.advance();
        let mut value = String::new();
        while !self.at_end() && !matches!(self.current(), b'"' | b'\n' | b'\r') {
            value.push(self.advance_utf8_char());
        }
        let closed = !self.at_end() && self.current() == b'"';
        if closed {
            self.advance();
        }
        let replacement = format!("'{}'", value.replace('\'', "''"));
        self.report_foreign_token(
            "Double-quoted string is not FPAS syntax",
            &replacement,
            "Strings use single quotes; an apostrophe inside is doubled.",
            start,
            closed.then_some(if value.contains('\\') {
                Applicability::NeedsReview
            } else {
                Applicability::MachineApplicable
            }),
        );
        let (so, sl, sc) = start;
        self.push_tok(Token::Str(value), so, sl, sc);
    }
}
