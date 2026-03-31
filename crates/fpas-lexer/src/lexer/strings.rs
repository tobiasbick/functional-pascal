use super::Lexer;
use crate::Token;
use fpas_diagnostics::codes::{
    LEX_INVALID_CHARACTER_CODE_LITERAL, LEX_UNTERMINATED_STRING_LITERAL,
};

impl Lexer<'_> {
    pub(super) fn scan_string_literal(&mut self) {
        let (so, sl, sc) = self.span_here();
        let mut value = String::new();

        loop {
            if self.at_end() {
                break;
            }
            match self.current() {
                b'\'' => self.scan_quoted_part(&mut value),
                b'#' => self.scan_char_code_part(&mut value),
                _ => break,
            }
        }

        self.push_tok(Token::Str(value), so, sl, sc);
    }

    pub(super) fn scan_quoted_part(&mut self, buf: &mut String) {
        let (so, sl, sc) = self.span_here();
        self.advance();

        loop {
            if self.at_end() {
                self.push_err(
                    LEX_UNTERMINATED_STRING_LITERAL,
                    "Unterminated string literal",
                    "Add a closing single quote. Use `''` inside a string to write a literal apostrophe.",
                    so, sl, sc,
                );
                return;
            }

            if self.current() == b'\'' {
                if self.peek_at(1) == Some(b'\'') {
                    buf.push('\'');
                    self.advance();
                    self.advance();
                } else {
                    self.advance();
                    return;
                }
            } else {
                buf.push(self.advance_utf8_char());
            }
        }
    }

    pub(super) fn scan_char_code_part(&mut self, buf: &mut String) {
        let (so, sl, sc) = self.span_here();
        self.advance();

        if self.at_end() || !self.current().is_ascii_digit() {
            self.push_err(
                LEX_INVALID_CHARACTER_CODE_LITERAL,
                "Expected decimal number after `#`",
                "Write digits after `#` in the range 0..255, for example `#65` for `A`.",
                so,
                sl,
                sc,
            );
            return;
        }

        let digits = self.consume_decimal_digits();
        let code: u32 = digits.parse().unwrap_or(u32::MAX);

        if code > 255 {
            self.push_err(
                LEX_INVALID_CHARACTER_CODE_LITERAL,
                &format!("Character code {code} is out of range"),
                "Use a character code between 0 and 255.",
                so,
                sl,
                sc,
            );
            buf.push('\u{FFFD}');
        } else {
            buf.push(code as u8 as char);
        }
    }
}
