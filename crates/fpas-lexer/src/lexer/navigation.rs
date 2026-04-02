use super::Lexer;
use crate::{Span, Token, error::lex_error};

impl Lexer<'_> {
    pub(super) const fn at_end(&self) -> bool {
        self.pos >= self.src.len()
    }

    pub(super) fn current(&self) -> u8 {
        self.src[self.pos]
    }

    pub(super) fn peek_at(&self, offset: usize) -> Option<u8> {
        self.src.get(self.pos + offset).copied()
    }

    pub(super) fn advance(&mut self) -> u8 {
        let ch = self.src[self.pos];
        self.pos += 1;
        match ch {
            b'\n' => {
                self.line += 1;
                self.col = 1;
            }
            b'\r' => {
                if self.pos >= self.src.len() || self.src[self.pos] != b'\n' {
                    self.line += 1;
                    self.col = 1;
                }
            }
            _ => {
                // Only count non-continuation bytes (0x80–0xBF) so that a
                // multi-byte UTF-8 codepoint advances the column by exactly one.
                if (ch & 0xC0) != 0x80 {
                    self.col += 1;
                }
            }
        }
        ch
    }

    /// Reads one Unicode scalar value from the current position, advances past
    /// all of its bytes, and returns the decoded [`char`].
    ///
    /// The source is always valid UTF-8 because the lexer is created from a
    /// `&str`.  Column tracking uses [`advance`][Self::advance] internally, so
    /// multi-byte codepoints increment the column counter by exactly one.
    ///
    /// # Panics
    ///
    /// Panics if called at end of input.
    pub(super) fn advance_utf8_char(&mut self) -> char {
        let remaining = match std::str::from_utf8(&self.src[self.pos..]) {
            Ok(remaining) => remaining,
            Err(_) => unreachable!("lexer source is always valid UTF-8"),
        };
        let Some(ch) = remaining.chars().next() else {
            unreachable!("advance_utf8_char called past end of input");
        };
        for _ in 0..ch.len_utf8() {
            self.advance();
        }
        ch
    }

    pub(super) const fn span_here(&self) -> (usize, u32, u32) {
        (self.pos, self.line, self.col)
    }

    pub(super) const fn make_span(
        &self,
        start_offset: usize,
        start_line: u32,
        start_col: u32,
    ) -> Span {
        Span {
            offset: start_offset,
            length: self.pos - start_offset,
            line: start_line,
            column: start_col,
            source_id: 0,
        }
    }

    pub(super) fn push_tok(&mut self, token: Token, so: usize, sl: u32, sc: u32) {
        let span = self.make_span(so, sl, sc);
        self.tokens.push(crate::SpannedToken { token, span });
    }

    pub(super) fn push_err(
        &mut self,
        code: fpas_diagnostics::DiagnosticCode,
        message: &str,
        hint: &str,
        so: usize,
        sl: u32,
        sc: u32,
    ) {
        let span = self.make_span(so, sl, sc);
        self.errors.push(lex_error(code, message, hint, span));
    }

    pub(super) fn emit_single(&mut self, token: Token) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        self.push_tok(token, so, sl, sc);
    }
}
