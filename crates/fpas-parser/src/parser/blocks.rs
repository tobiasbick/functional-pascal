//! Named block endings and recovery that preserves enclosing construct boundaries.
//!
//! **Documentation:** `docs/pascal/language/control-flow/README.md`.

use super::{Parser, token_display};
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    /// Consumes a matching named closer without stealing an enclosing closer on error.
    pub(crate) fn expect_named_end(&mut self, kind: &Token) -> bool {
        if self.check(&Token::End) && self.peek_token() == kind {
            self.advance();
            self.advance();
            return true;
        }
        let name = token_display(kind);
        let mut hint = format!("Close this construct with `end {name};`.");
        if kind == &Token::If {
            hint.push_str(" `else if` starts a nested conditional with its own `end if;`; use `elsif` to continue the same conditional.");
        }
        if kind == &Token::Unit && self.check(&Token::Begin) {
            hint.push_str(" Unit files contain declarations only. Remove the main block.");
        }
        self.error_with_code(
            PARSE_EXPECTED_TOKEN,
            &format!(
                "Expected `end {name}` to close this construct, found `{}`",
                token_display(self.current_token())
            ),
            &hint,
            self.current_span(),
        );
        // A bare legacy `end` has no owner keyword and can be consumed safely.
        // A named outer closer remains available to its owning production.
        if self.check(&Token::End)
            && matches!(
                self.peek_token(),
                Token::Semicolon | Token::Dot | Token::Eof
            )
        {
            self.advance();
        }
        false
    }
}
