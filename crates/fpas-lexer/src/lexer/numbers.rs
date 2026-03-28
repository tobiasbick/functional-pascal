use super::Lexer;
use crate::Token;
use fpas_diagnostics::codes::{
    LEX_INTEGER_LITERAL_OVERFLOW, LEX_INVALID_HEXADECIMAL_LITERAL, LEX_INVALID_NUMERIC_EXPONENT,
    LEX_REAL_LITERAL_OVERFLOW,
};

impl Lexer<'_> {
    pub(super) fn scan_number(&mut self) {
        let (so, sl, sc) = self.span_here();
        let int_part = self.consume_decimal_digits();

        if !self.at_end()
            && self.current() == b'.'
            && self.peek_at(1).is_some_and(|c| c.is_ascii_digit())
        {
            self.advance();
            let frac_part = self.consume_decimal_digits();
            let exp_part = match self.maybe_scan_exponent() {
                Ok(exp_part) => exp_part,
                Err(()) => return,
            };

            let text = format!("{int_part}.{frac_part}{exp_part}");
            match text.parse::<f64>() {
                Ok(v) => self.push_tok(Token::Real(v), so, sl, sc),
                Err(_) => self.push_err(
                    LEX_REAL_LITERAL_OVERFLOW,
                    "Real literal is out of range",
                    "Use a smaller value or exponent so it fits in a 64-bit floating-point number.",
                    so,
                    sl,
                    sc,
                ),
            }
        } else {
            match int_part.parse::<i64>() {
                Ok(v) => self.push_tok(Token::Integer(v), so, sl, sc),
                Err(_) => self.push_err(
                    LEX_INTEGER_LITERAL_OVERFLOW,
                    "Integer literal is too large",
                    "Use a value up to 9223372036854775807, or rewrite the literal as a `Real` value.",
                    so,
                    sl,
                    sc,
                ),
            }
        }
    }

    pub(super) fn scan_hex_integer(&mut self) {
        let (so, sl, sc) = self.span_here();
        self.advance();

        if self.at_end() || !self.current().is_ascii_hexdigit() {
            self.push_err(
                LEX_INVALID_HEXADECIMAL_LITERAL,
                "Expected hexadecimal digit after `$`",
                "Write at least one hexadecimal digit after `$`, for example `$FF` or `$1A`.",
                so,
                sl,
                sc,
            );
            return;
        }

        let hex_str = self.consume_hex_digits();
        match i64::from_str_radix(&hex_str, 16) {
            Ok(v) => self.push_tok(Token::Integer(v), so, sl, sc),
            Err(_) => self.push_err(
                LEX_INVALID_HEXADECIMAL_LITERAL,
                "Hex literal is too large",
                "Use a hexadecimal value up to `$7FFFFFFFFFFFFFFF`.",
                so,
                sl,
                sc,
            ),
        }
    }

    pub(super) fn consume_decimal_digits(&mut self) -> String {
        let mut digits = String::new();
        if !self.at_end() && self.current().is_ascii_digit() {
            digits.push(self.advance() as char);
        }

        while !self.at_end() {
            if self.current().is_ascii_digit() {
                digits.push(self.advance() as char);
            } else if self.current() == b'_' && self.peek_at(1).is_some_and(|c| c.is_ascii_digit())
            {
                self.advance();
                digits.push(self.advance() as char);
            } else {
                break;
            }
        }

        digits
    }

    pub(super) fn consume_hex_digits(&mut self) -> String {
        let mut digits = String::new();
        if !self.at_end() && self.current().is_ascii_hexdigit() {
            digits.push(self.advance() as char);
        }

        while !self.at_end() {
            if self.current().is_ascii_hexdigit() {
                digits.push(self.advance() as char);
            } else if self.current() == b'_'
                && self.peek_at(1).is_some_and(|c| c.is_ascii_hexdigit())
            {
                self.advance();
                digits.push(self.advance() as char);
            } else {
                break;
            }
        }

        digits
    }

    pub(super) fn maybe_scan_exponent(&mut self) -> Result<String, ()> {
        if self.at_end() || !matches!(self.current(), b'e' | b'E') {
            return Ok(String::new());
        }

        let (so, sl, sc) = self.span_here();
        let mut exp = String::new();
        exp.push(self.advance() as char);

        if !self.at_end() && matches!(self.current(), b'+' | b'-') {
            exp.push(self.advance() as char);
        }

        let digits = self.consume_decimal_digits();
        if digits.is_empty() {
            self.push_err(
                LEX_INVALID_NUMERIC_EXPONENT,
                "Invalid numeric exponent",
                "Add at least one digit after `e` or `E`, for example `1.0e3` or `1.0e-3`.",
                so,
                sl,
                sc,
            );
            return Err(());
        }

        exp.push_str(&digits);
        Ok(exp)
    }
}
