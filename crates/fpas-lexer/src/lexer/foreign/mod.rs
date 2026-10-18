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
use fpas_diagnostics::codes::LEX_FOREIGN_TOKEN;
use fpas_diagnostics::{Applicability, ForeignHabit, SourceSpan};

impl Lexer<'_> {
    /// Reports one foreign token written from the given start up to the current position.
    ///
    /// With `fix`, the diagnostic carries an edit that replaces the written text with
    /// `replacement`, spaced apart from adjacent identifier characters.
    fn report_foreign_token(
        &mut self,
        message: &str,
        replacement: &str,
        explanation: &str,
        start: (usize, u32, u32),
        fix: Option<Applicability>,
    ) {
        let (so, sl, sc) = start;
        let span = self.make_span(so, sl, sc);
        let found = self.source.get(so..self.pos).unwrap_or_default().to_owned();
        let habit = ForeignHabit {
            found: &found,
            replacement,
            explanation,
        };
        let mut diagnostic =
            habit.attach(lex_error(LEX_FOREIGN_TOKEN, message, String::new(), span));
        if let Some(applicability) = fix {
            let new_text = self.spaced_replacement(so, replacement);
            diagnostic = diagnostic.with_fix(habit.fix(
                applicability,
                SourceSpan::try_from(span).ok(),
                new_text,
            ));
        }
        self.errors.push(diagnostic);
    }

    /// Adds a space on each side of `replacement` where a word would otherwise merge with an
    /// adjacent identifier character, as in `A&&B` → `A and B`.
    fn spaced_replacement(&self, start: usize, replacement: &str) -> String {
        let bytes = self.source.as_bytes();
        let is_word =
            |byte: Option<&u8>| byte.is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_');
        let starts_word = is_word(replacement.as_bytes().first());
        let ends_word = is_word(replacement.as_bytes().last());
        let before = start.checked_sub(1).and_then(|index| bytes.get(index));
        let mut text = String::new();
        if starts_word && is_word(before) {
            text.push(' ');
        }
        text.push_str(replacement);
        if ends_word && is_word(bytes.get(self.pos)) {
            text.push(' ');
        }
        text
    }
}
