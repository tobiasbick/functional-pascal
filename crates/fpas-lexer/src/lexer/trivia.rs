use super::Lexer;
use crate::SourceComment;

impl Lexer<'_> {
    pub(super) fn skip_trivia(&mut self) {
        loop {
            self.skip_whitespace();
            if self.at_end() {
                break;
            }
            match self.current() {
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

    fn record_comment(&mut self, start_offset: usize, start_line: u32, start_col: u32) {
        let span = self.make_span(start_offset, start_line, start_col);
        self.comments.push(SourceComment { span });
    }

    pub(super) fn skip_line_comment(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();
        self.advance();
        while !self.at_end() && self.current() != b'\n' && self.current() != b'\r' {
            self.advance();
        }
        self.record_comment(so, sl, sc);
    }
}
