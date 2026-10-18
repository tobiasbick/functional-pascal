//! Exceptions from Ada and Delphi: `exception when … =>` handlers and `raise` statements.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/error-handling/README.md`

use super::super::Parser;
use fpas_diagnostics::ForeignHabit;
use fpas_diagnostics::codes::PARSE_FOREIGN_SYNTAX;
use fpas_lexer::Token;

impl Parser {
    /// True at an Ada exception handler `exception when` inside a statement list.
    pub(in crate::parser) fn at_ada_exception_handler(&self) -> bool {
        Self::is_ident(self.current_token(), "exception") && self.peek_token() == &Token::When
    }

    /// Skips an Ada exception handler up to the `end` that closes its block and reports it.
    pub(in crate::parser) fn skip_ada_exception_handler(&mut self) {
        let start = self.current_span();
        let mut depth = 0_usize;
        while !self.at_end() {
            match self.current_token() {
                Token::Begin => depth += 1,
                // `end if`, `end case` and `end loop` close constructs inside a handler.
                Token::End if self.at_construct_end() => {
                    self.advance();
                }
                Token::End if depth == 0 => break,
                Token::End => depth -= 1,
                _ => {}
            }
            self.advance();
        }
        let written = self.span_from(start);
        let found = self
            .source_text(written)
            .split_whitespace()
            .take(4)
            .collect::<Vec<_>>()
            .join(" ");
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "Ada exception handlers are not FPAS syntax",
            ForeignHabit {
                found: &found,
                replacement: "a `Result of (T, E)` handled with `case` or propagated with `try Expr`",
                explanation: "FPAS has no exceptions: an operation that can fail returns `Result of (T, E)`, and `panic` stops the program on a bug.",
            },
            written,
            None,
        );
    }

    /// True at Ada `raise Name` or Delphi `raise Exception.Create(…)`.
    pub(in crate::parser) fn at_raise_statement(&self) -> bool {
        Self::is_ident(self.current_token(), "raise")
            && matches!(self.peek_token(), Token::Ident(_))
    }

    /// Skips a `raise` statement through its `;` and reports it with the FPAS failure forms.
    pub(in crate::parser) fn skip_raise_statement(&mut self) {
        let start = self.current_span();
        let mut message = None;
        let mut depth = 0_usize;
        while !self.at_end() {
            match self.current_token() {
                Token::LParen => depth += 1,
                Token::RParen if depth > 0 => depth -= 1,
                Token::Semicolon | Token::End if depth == 0 => break,
                Token::Str(text) if message.is_none() => message = Some(text.clone()),
                _ if depth == 0 && self.is_stmt_list_end() => break,
                _ => {}
            }
            self.advance();
        }
        let written = self.span_from(start);
        self.eat(&Token::Semicolon);
        let found = self.source_text(written).to_owned();
        let text = message
            .unwrap_or_else(|| "…".to_owned())
            .replace('\'', "''");
        let replacement = format!("return Error('{text}'); or panic('{text}');");
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "`raise` is not FPAS syntax",
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation: "FPAS has no exceptions: return `Error(…)` from a routine whose result is `Result of (T, E)` when the failure is expected, and `panic` on a bug.",
            },
            written,
            None,
        );
    }

    fn at_construct_end(&self) -> bool {
        matches!(
            self.peek_token(),
            Token::If | Token::Case | Token::For | Token::While | Token::Record
        ) || Self::is_ident(self.peek_token(), "loop")
            || Self::is_ident(self.peek_token(), "select")
    }
}
