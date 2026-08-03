use super::Lexer;
use crate::{CommentStyle, SourceComment};
use fpas_diagnostics::codes::{LEX_UNTERMINATED_BRACE_COMMENT, LEX_UNTERMINATED_PAREN_COMMENT};

impl Lexer<'_> {
    pub(super) fn skip_trivia(&mut self) {
        loop {
            self.skip_whitespace();
            if self.at_end() {
                break;
            }
            match self.current() {
                b'{' if self.is_directive_after_brace() => break,
                b'{' => self.skip_brace_comment(),
                b'(' if self.peek_at(1) == Some(b'*') => self.skip_paren_comment(),
                b'/' if self.peek_at(1) == Some(b'/') => self.skip_line_comment(),
                _ => break,
            }
        }
    }

    pub(super) fn skip_whitespace(&mut self) {
        while !self.at_end() {
            let Some(ch) = self.remaining_str().chars().next() else {
                break;
            };
            // Only the first scalar may be a UTF-8 BOM. A later U+FEFF must reach the
            // unexpected-character path instead of silently separating tokens.
            if ch == '\u{FEFF}' && self.pos != 0 {
                break;
            }
            if ch != '\u{FEFF}' && !ch.is_whitespace() {
                break;
            }
            self.advance_utf8_char();
        }
    }

    fn record_comment(
        &mut self,
        style: CommentStyle,
        start_offset: usize,
        start_line: u32,
        start_col: u32,
    ) {
        let span = self.make_span(start_offset, start_line, start_col);
        self.comments.push(SourceComment { style, span });
    }

    /// Scans past `opener_advances` bytes, then until a closing `}`.
    ///
    /// Returns `true` when the closing brace was found and consumed.
    pub(super) fn scan_brace_body(&mut self, opener_advances: usize) -> bool {
        for _ in 0..opener_advances {
            self.advance();
        }
        while !self.at_end() {
            if self.current() == b'}' {
                self.advance();
                return true;
            }
            self.advance();
        }
        false
    }

    pub(super) fn skip_brace_comment(&mut self) {
        let (so, sl, sc) = self.span_here();
        if self.scan_brace_body(1) {
            self.record_comment(CommentStyle::Brace, so, sl, sc);
            return;
        }

        self.push_err(
            LEX_UNTERMINATED_BRACE_COMMENT,
            "Unterminated comment starting with `{`",
            "Add a closing `}` before end of file. Brace comments do not nest.",
            so,
            sl,
            sc,
        );
    }

    pub(super) fn skip_paren_comment(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        self.advance();

        while !self.at_end() {
            if self.current() == b'*' && self.peek_at(1) == Some(b')') {
                self.advance();
                self.advance();
                self.record_comment(CommentStyle::Paren, so, sl, sc);
                return;
            }
            self.advance();
        }

        self.push_err(
            LEX_UNTERMINATED_PAREN_COMMENT,
            "Unterminated comment starting with `(*`",
            "Add a closing `*)` before end of file.",
            so,
            sl,
            sc,
        );
    }

    pub(super) fn skip_line_comment(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        self.advance();
        // `///` is a doc line; bare `//` at EOF must not call `current()`.
        let style = if !self.at_end() && self.current() == b'/' {
            CommentStyle::DocLine
        } else {
            CommentStyle::Line
        };
        while !self.at_end() && self.current() != b'\n' && self.current() != b'\r' {
            self.advance();
        }
        self.record_comment(style, so, sl, sc);
    }
}
