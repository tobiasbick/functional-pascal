//! Recognition of spellings from other languages (Delphi, Free Pascal, Ada, C family) that the
//! lexer rejects with a hint showing the FPAS form. Where FPAS has an equivalent token, the lexer
//! emits it so the parser continues as if the FPAS form had been written.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

mod attributes;
mod comments;
mod operators;
mod strings;

use super::Lexer;
use crate::error::lex_error;
use fpas_diagnostics::ForeignHabit;
use fpas_diagnostics::codes::LEX_FOREIGN_TOKEN;

impl Lexer<'_> {
    /// Reports one foreign token written from the given start up to the current position.
    fn report_foreign_token(
        &mut self,
        message: &str,
        replacement: &str,
        explanation: &str,
        start: (usize, u32, u32),
    ) {
        let (so, sl, sc) = start;
        let span = self.make_span(so, sl, sc);
        let found = self.source.get(so..self.pos).unwrap_or_default().to_owned();
        let habit = ForeignHabit {
            found: &found,
            replacement,
            explanation,
        };
        self.errors
            .push(habit.attach(lex_error(LEX_FOREIGN_TOKEN, message, String::new(), span)));
    }
}
