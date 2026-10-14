//! Quoted strings and decimal Unicode scalar codes share adjacent-element decoding.
//! See `docs/pascal/language/basics/primitive-types.md`.

use super::Lexer;
use crate::Token;
use fpas_diagnostics::codes::{
    LEX_INVALID_CHARACTER_CODE_LITERAL, LEX_UNTERMINATED_STRING_LITERAL,
};

const CHARACTER_CODE_HINT: &str = "Use a decimal Unicode scalar in 0..1114111, excluding 55296..57343; for example `#65` for `A` or `#8364` for `€`.";

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
                b'#' => {
                    // Stop concatenating after an invalid `#` so later quoted
                    // parts become separate tokens instead of a forged string.
                    if !self.scan_char_code_part(&mut value) {
                        break;
                    }
                }
                _ => break,
            }
        }

        self.push_tok(Token::Str(value), so, sl, sc);
    }

    fn scan_quoted_part(&mut self, buf: &mut String) {
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

    /// Appends one decimal `#N` Unicode scalar. Returns `false` when the code is invalid
    /// (caller should stop the surrounding string concatenation).
    fn scan_char_code_part(&mut self, buf: &mut String) -> bool {
        let (so, sl, sc) = self.span_here();
        self.advance();

        if self.at_end() || !self.current().is_ascii_digit() {
            self.push_err(
                LEX_INVALID_CHARACTER_CODE_LITERAL,
                "Expected decimal number after `#`",
                CHARACTER_CODE_HINT,
                so,
                sl,
                sc,
            );
            return false;
        }

        let digits = self.consume_decimal_digits();
        let code: u32 = match digits.parse() {
            Ok(v) => v,
            Err(_) => {
                self.push_err(
                    LEX_INVALID_CHARACTER_CODE_LITERAL,
                    "Character code literal is too large to represent",
                    CHARACTER_CODE_HINT,
                    so,
                    sl,
                    sc,
                );
                return false;
            }
        };

        let Some(character) = char::from_u32(code) else {
            self.push_err(
                LEX_INVALID_CHARACTER_CODE_LITERAL,
                &format!("Character code {code} is not a valid Unicode scalar value"),
                CHARACTER_CODE_HINT,
                so,
                sl,
                sc,
            );
            return false;
        };
        buf.push(character);
        true
    }
}
