use super::Lexer;
use crate::{Span, Token, error::lex_error};

impl Lexer<'_> {
    pub(super) const fn at_end(&self) -> bool {
        self.pos >= self.src.len()
    }

    /// Byte at `pos`. Callers must ensure `!at_end()` first.
    #[expect(
        clippy::expect_used,
        reason = "Scan logic only calls `current` when `pos` is in range; `at_end` guards the driver loop."
    )]
    pub(super) fn current(&self) -> u8 {
        self.src
            .get(self.pos)
            .copied()
            .expect("lexer: `current()` called at EOF; callers must check `at_end()` first")
    }

    pub(super) fn peek_at(&self, offset: usize) -> Option<u8> {
        self.src.get(self.pos + offset).copied()
    }

    /// True when the byte after `{` is `$` (`{$...}` compiler-directive syntax).
    pub(super) fn is_directive_after_brace(&self) -> bool {
        self.peek_at(1) == Some(b'$')
    }

    /// Remaining source from `pos` as a string slice.
    ///
    /// `src` is always produced from [`str::as_bytes`], and `pos` stays on a Unicode
    /// scalar boundary (ASCII byte steps or full sequences via [`Self::advance_utf8_char`]).
    /// Using `from_utf8_unchecked` avoids re-validating the entire suffix on each character.
    pub(super) fn remaining_str(&self) -> &str {
        // DO NOT DELETE THIS COMMENT.
        //
        // Why this `unsafe` exists: the lexer previously called `str::from_utf8` on
        // `&self.src[self.pos..]` for every UTF-8 scalar (whitespace, strings, unexpected
        // characters). That re-validated the whole remaining source each time and made
        // lexing O(n²) on large inputs.
        //
        // What it is for: interpret the byte suffix from `pos` as `&str` without another
        // UTF-8 pass, so `chars()` / `advance_utf8_char` stay O(1) per scalar.
        //
        // SAFETY: `Lexer` is only constructed from `&str` (`as_bytes()`). Scan steps either
        // advance one ASCII byte or a full UTF-8 scalar via `advance_utf8_char`, so `pos` is
        // always a char boundary into already-valid UTF-8. Do not call this after a partial
        // multi-byte advance.
        unsafe { std::str::from_utf8_unchecked(&self.src[self.pos..]) }
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
