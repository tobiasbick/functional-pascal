//! Ada main program structure: `program Name is … begin … end Name;`.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/program-structure/README.md`

use super::super::Parser;
use fpas_diagnostics::codes::PARSE_FOREIGN_SYNTAX;
use fpas_diagnostics::{Applicability, Fix, ForeignHabit, SourceSpan};
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
        let index = self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "Ada main program structure is not FPAS syntax",
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation: "An FPAS program header ends with `;` and the program closes with `end.`.",
            },
            written,
            None,
        );
        // `program Name is` becomes `program Name;`; the ending edit follows once
        // `eat_ada_program_ending` finds `end Name;`.
        let header = format!("program {name};");
        let fix = SourceSpan::try_from(written).ok().and_then(|span| {
            Fix::replace(
                "Rewrite as an FPAS program",
                Applicability::MachineApplicable,
                span,
                header,
            )
        });
        self.add_habit_fix(index, fix);
        self.ada_program_fix = index.map(|index| (index, "."));
        true
    }

    /// Consumes the Ada program ending `end Name;` after a reported Ada program header.
    pub(in crate::parser) fn eat_ada_program_ending(&mut self, name: &str) -> bool {
        if self.check(&Token::End)
            && matches!(self.peek_token(), Token::Ident(ending) if ending.eq_ignore_ascii_case(name))
        {
            let end = self.advance().span;
            self.advance();
            self.eat(&Token::Semicolon);
            // `end Name;` becomes `end.`: the edit starts right after `end`.
            let after_end = Span {
                offset: end.offset + end.length,
                length: 0,
                column: end.column + u32::try_from(end.length).unwrap_or(0),
                ..end
            };
            let ending = self.span_from(after_end);
            let pending = self.ada_program_fix.take();
            self.complete_ada_fix(pending, Some(ending));
            true
        } else {
            let pending = self.ada_program_fix.take();
            self.complete_ada_fix(pending, None);
            false
        }
    }
}
