//! Operators from Delphi, Ada and the C family: `&&`, `||`, `!`, `!=`, `==`, `/=`, `=>`, `&`, the
//! Pascal address operator `@` and pointer marker `^`.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/basics/operators.md`

use crate::Token;
use crate::lexer::Lexer;
use fpas_diagnostics::Applicability;

impl Lexer<'_> {
    /// True at an operator from another language.
    pub(in crate::lexer) fn at_foreign_operator(&self) -> bool {
        matches!(
            (self.current(), self.peek_at(1)),
            (b'&', _)
                | (b'|', Some(b'|'))
                | (b'!', _)
                | (b'=', Some(b'=' | b'>'))
                | (b'/', Some(b'='))
                | (b'@', _)
                | (b'^', _)
        )
    }

    /// Reports a foreign operator and emits its FPAS token, if FPAS has one.
    pub(in crate::lexer) fn scan_foreign_operator(&mut self) {
        let start = self.span_here();
        let (token, replacement, message, explanation) = match (self.current(), self.peek_at(1)) {
            (b'&', Some(b'&')) => (
                Some(Token::And),
                "and",
                "C-family `&&` is not FPAS syntax",
                "Logical operators are words.",
            ),
            (b'&', _) => (
                Some(Token::Plus),
                "+",
                "Ada `&` concatenation is not FPAS syntax",
                "Strings concatenate with `+`.",
            ),
            (b'|', _) => (
                Some(Token::Or),
                "or",
                "C-family `||` is not FPAS syntax",
                "Logical operators are words.",
            ),
            (b'!', Some(b'=')) => (
                Some(Token::NotEqual),
                "<>",
                "C-family `!=` is not FPAS syntax",
                "Inequality is written `<>`.",
            ),
            (b'!', _) => (
                Some(Token::Not),
                "not",
                "C-family `!` is not FPAS syntax",
                "Logical operators are words.",
            ),
            (b'=', Some(b'=')) => (
                Some(Token::Equal),
                "=",
                "C-family `==` is not FPAS syntax",
                "Equality is a single `=`; assignment is `:=`.",
            ),
            (b'=', _) => (
                Some(Token::ColonAssign),
                ":=",
                "Ada `=>` is not FPAS syntax",
                "Named arguments are written `Name := Value`, as in `F(X := 1)`.",
            ),
            (b'/', _) => (
                Some(Token::NotEqual),
                "<>",
                "Ada `/=` is not FPAS syntax",
                "Inequality is written `<>`.",
            ),
            (b'@', _) => (
                None,
                "F",
                "Pascal address operator `@` is not FPAS syntax",
                "A routine name is already a callable value; FPAS has no pointers.",
            ),
            _ => (
                None,
                "Option of T",
                "Pascal pointer `^` is not FPAS syntax",
                "FPAS has no pointers; use records, arrays, or `Option of T` for an optional value.",
            ),
        };
        let width = match (self.current(), self.peek_at(1)) {
            (b'!', Some(b'=')) | (b'&', Some(b'&')) | (b'|' | b'=' | b'/', _) => 2,
            _ => 1,
        };
        for _ in 0..width {
            self.advance();
        }
        let fix = token.as_ref().map(|_| Applicability::MachineApplicable);
        self.report_foreign_token(message, replacement, explanation, start, fix);
        if let Some(token) = token {
            let (so, sl, sc) = start;
            self.push_tok(token, so, sl, sc);
        }
    }
}
