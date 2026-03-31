use super::Lexer;
use fpas_diagnostics::codes::{LEX_UNTERMINATED_BRACE_COMMENT, LEX_UNTERMINATED_PAREN_COMMENT};

impl Lexer<'_> {
    pub(super) fn skip_trivia(&mut self) {
        loop {
            self.skip_whitespace();
            if self.at_end() {
                break;
            }
            match self.current() {
                // A `{$...}` sequence is not a brace comment; `scan_token` reports an error.
                b'{' if self.peek_at(1) == Some(b'$') => break,
                b'{' => self.skip_brace_comment(),
                b'(' if self.peek_at(1) == Some(b'*') => self.skip_paren_comment(),
                b'/' if self.peek_at(1) == Some(b'/') => self.skip_line_comment(),
                _ => break,
            }
        }
    }

    pub(super) fn skip_whitespace(&mut self) {
        while !self.at_end() && self.current().is_ascii_whitespace() {
            self.advance();
        }
    }

    pub(super) fn skip_brace_comment(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();

        while !self.at_end() {
            if self.current() == b'}' {
                self.advance();
                return;
            }
            self.advance();
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
        self.advance();
        self.advance();
        while !self.at_end() && self.current() != b'\n' && self.current() != b'\r' {
            self.advance();
        }
    }
}
