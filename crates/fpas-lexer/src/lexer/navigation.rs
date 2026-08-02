use super::Lexer;
use crate::{SourcePosition, Span, Token, error::lex_error};

impl Lexer<'_> {
    pub(super) const fn at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    const fn bytes(&self) -> &[u8] {
        self.source.as_bytes()
    }

    /// Byte at `pos`. Callers must ensure `!at_end()` first.
    #[expect(
        clippy::expect_used,
        reason = "Scan logic only calls `current` when `pos` is in range; `at_end` guards the driver loop."
    )]
    pub(super) fn current(&self) -> u8 {
        self.bytes()
            .get(self.pos)
            .copied()
            .expect("lexer: `current()` called at EOF; callers must check `at_end()` first")
    }

    pub(super) fn peek_at(&self, offset: usize) -> Option<u8> {
        self.bytes().get(self.pos + offset).copied()
    }

    /// True when the byte after `{` is `$` (`{$...}` compiler-directive syntax).
    pub(super) fn is_directive_after_brace(&self) -> bool {
        self.peek_at(1) == Some(b'$')
    }

    /// Remaining source from `pos` as a string slice.
    ///
    /// Direct slicing checks the UTF-8 boundary in constant time without re-validating
    /// the remaining source.
    ///
    /// # Panics
    ///
    /// Panics if `pos` is not on a UTF-8 character boundary.
    pub(super) fn remaining_str(&self) -> &str {
        &self.source[self.pos..]
    }

    pub(super) fn advance(&mut self) -> u8 {
        let ch = self.bytes()[self.pos];
        self.pos += 1;
        match ch {
            b'\n' => {
                self.line += 1;
                self.col = 1;
            }
            b'\r' => {
                if self.pos >= self.source.len() || self.bytes()[self.pos] != b'\n' {
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
    #[allow(clippy::expect_used)] // EOF without a scalar is a caller bug.
    pub(super) fn advance_utf8_char(&mut self) -> char {
        let ch = self
            .remaining_str()
            .chars()
            .next()
            .expect("advance_utf8_char called past end of input");
        for _ in 0..ch.len_utf8() {
            self.advance();
        }
        ch
    }

    pub(super) const fn span_here(&self) -> (usize, u32, u32) {
        (self.pos, self.line, self.col)
    }

    const fn position_here(&self) -> SourcePosition {
        SourcePosition {
            offset: self.pos,
            line: self.line,
            column: self.col,
        }
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
            source_id: self.source_id,
        }
    }

    pub(super) fn push_tok(&mut self, token: Token, so: usize, sl: u32, sc: u32) {
        let span = self.make_span(so, sl, sc);
        let end = self.position_here();
        self.tokens.push(crate::SpannedToken { token, span, end });
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
