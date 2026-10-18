//! Ada main program structure: `program Name is … begin … end Name;`.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/program-structure/README.md`

use super::super::Parser;
use fpas_diagnostics::ForeignHabit;
use fpas_diagnostics::codes::PARSE_FOREIGN_SYNTAX;
use fpas_lexer::{Span, Token};

impl Parser {
    /// Consumes Ada `is` after the program name and reports the Ada program structure once.
    /// Returns whether it was found.
    pub(in crate::parser) fn eat_ada_program_header(&mut self, start: Span, name: &str) -> bool {
        if !self.eat(&Token::Is) {
            return false;
        }
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        let replacement = format!("program {name}; … begin … end.");
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "Ada main program structure is not FPAS syntax",
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation: "An FPAS program header ends with `;` and the program closes with `end.`.",
            },
            written,
        );
        true
    }

    /// Consumes the Ada program ending `end Name;` after a reported Ada program header.
    pub(in crate::parser) fn eat_ada_program_ending(&mut self, name: &str) -> bool {
        if self.check(&Token::End)
            && matches!(self.peek_token(), Token::Ident(ending) if ending.eq_ignore_ascii_case(name))
        {
            self.advance();
            self.advance();
            self.eat(&Token::Semicolon);
            true
        } else {
            false
        }
    }
}
