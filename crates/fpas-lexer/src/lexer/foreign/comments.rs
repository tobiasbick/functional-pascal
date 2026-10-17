//! Recovery for comment forms from Pascal (`{…}`, `(*…*)`) and Ada (`--`).
//!
//! **Documentation:** `docs/pascal/language/basics/comments.md`

use crate::lexer::Lexer;
use fpas_diagnostics::codes::LEX_INVALID_COMMENT_FORM;

const COMMENT_HINT: &str = "Use `// comment`. For multiple lines, prefix each line with `//`.";

impl Lexer<'_> {
    /// Consumes `{...}` or the remaining source and reports one actionable diagnostic.
    pub(in crate::lexer) fn scan_invalid_brace_comment(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        while !self.at_end() {
            let closes = self.current() == b'}';
            self.advance();
            if closes {
                break;
            }
        }
        self.push_err(
            LEX_INVALID_COMMENT_FORM,
            "`{...}` is not valid comment syntax",
            COMMENT_HINT,
            so,
            sl,
            sc,
        );
    }

    /// Consumes `(*...*)` or the remaining source and reports one actionable diagnostic.
    pub(in crate::lexer) fn scan_invalid_paren_comment(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        self.advance();
        while !self.at_end() {
            if self.current() == b'*' && self.peek_at(1) == Some(b')') {
                self.advance();
                self.advance();
                break;
            }
            self.advance();
        }
        self.push_err(
            LEX_INVALID_COMMENT_FORM,
            "`(*...*)` is not valid comment syntax",
            COMMENT_HINT,
            so,
            sl,
            sc,
        );
    }
}

impl Lexer<'_> {
    /// True at an Ada `--` comment: at the start of a line or after a statement terminator, so
    /// `X - -1` stays an expression.
    pub(in crate::lexer) fn at_ada_comment(&self) -> bool {
        if self.current() != b'-' || self.peek_at(1) != Some(b'-') {
            return false;
        }
        let before = self.source.get(..self.pos).unwrap_or_default();
        let line_start = before
            .rsplit('\n')
            .next()
            .unwrap_or_default()
            .chars()
            .all(|ch| ch == ' ' || ch == '\t' || ch == '\r');
        line_start
            || self
                .tokens
                .last()
                .is_some_and(|token| token.token == crate::Token::Semicolon)
    }

    /// Consumes an Ada `--` comment through the end of its line and reports it.
    pub(in crate::lexer) fn scan_ada_comment(&mut self) {
        let start = self.span_here();
        while !self.at_end() && !matches!(self.current(), b'\n' | b'\r') {
            self.advance_utf8_char();
        }
        let text = self
            .source
            .get(start.0 + 2..self.pos)
            .unwrap_or_default()
            .trim()
            .to_owned();
        let replacement = format!("// {text}");
        self.report_foreign_token(
            "Ada `--` comment is not FPAS syntax",
            &replacement,
            "Comments start with `//`.",
            start,
        );
    }
}
