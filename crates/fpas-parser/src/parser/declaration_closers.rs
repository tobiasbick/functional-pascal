//! Named declaration endings and recovery at enclosing declaration boundaries.
//!
//! **Documentation:** `docs/pascal/language/functions/declarations.md`,
//! `docs/pascal/program-structure/units.md` (from the repository root).

use super::{Parser, token_display};
use crate::error::parse_error;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    /// Parses a declaration while retaining its enclosing closer kinds for recovery.
    pub(super) fn with_declaration_closer<T>(
        &mut self,
        kind: Token,
        parse: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.declaration_closers.push(kind);
        let result = parse(self);
        self.declaration_closers.pop();
        result
    }

    /// Requires `end <kind>` and leaves the terminator to the owning declaration.
    ///
    /// Returns whether a closer was consumed. Enclosing named closers and the
    /// program's `end.` remain available to their owner when an inner closer is missing.
    pub(super) fn expect_declaration_end(&mut self, kind: &Token) -> bool {
        if self.check(&Token::End) && self.peek_token() == kind {
            self.advance();
            self.advance();
            return true;
        }

        let expected = format!("end {};", token_display(kind));
        let found = if self.check(&Token::End) {
            match self.peek_token() {
                Token::Semicolon => "end;".to_owned(),
                Token::Dot => "end.".to_owned(),
                Token::Eof => "end".to_owned(),
                token => format!("end {};", token_display(token)),
            }
        } else {
            token_display(self.current_token()).into_owned()
        };
        let mut hint = format!("Close this declaration with `{expected}`.");
        if kind == &Token::Unit && self.check(&Token::Begin) {
            hint.push_str(
                " Unit files contain declarations only. Remove trailing statements or blocks.",
            );
        }
        if !self.nesting_limit_reached {
            self.errors.push(
                parse_error(
                    PARSE_EXPECTED_TOKEN,
                    format!("Expected `{expected}`, found `{found}`"),
                    hint,
                    self.current_span(),
                )
                .with_expected_found(expected, found),
            );
        }

        if !self.check(&Token::End) || self.peek_token() == &Token::Dot {
            return false;
        }
        let enclosing = self
            .declaration_closers
            .iter()
            .rev()
            .skip(1)
            .any(|outer| outer == self.peek_token());
        if enclosing {
            return false;
        }
        if matches!(self.peek_token(), Token::Semicolon | Token::Eof) {
            self.advance();
            return true;
        }
        if matches!(
            self.peek_token(),
            Token::Function | Token::Procedure | Token::Record | Token::Enum | Token::Unit
        ) || self
            .tokens
            .get(self.pos + 2)
            .is_some_and(|token| token.token == Token::Semicolon)
        {
            self.advance();
            self.advance();
            return true;
        }
        false
    }

    /// Identifies a preserved outer ending after an inner declaration failed to close.
    pub(super) fn at_enclosing_declaration_end(&self) -> bool {
        self.check(&Token::End)
            && (self.peek_token() == &Token::Dot
                || self
                    .declaration_closers
                    .iter()
                    .any(|kind| kind == self.peek_token()))
    }
}
