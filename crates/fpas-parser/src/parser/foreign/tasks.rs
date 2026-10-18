//! Ada task declarations: `task Name;`, `task type Name is … end Name;` and
//! `task body Name is … end Name;`.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/concurrency/README.md`

use super::super::Parser;
use fpas_diagnostics::ForeignHabit;
use fpas_diagnostics::codes::PARSE_FOREIGN_SYNTAX;
use fpas_lexer::Token;

impl Parser {
    /// Skips one Ada task declaration at a declaration position and reports it.
    pub(in crate::parser) fn skip_ada_task(&mut self) {
        let start = self.current_span();
        self.advance();
        if !self.eat(&Token::Type) {
            self.eat_ident("body");
        }
        let name = match self.current_token() {
            Token::Ident(name) => name.clone(),
            _ => String::new(),
        };
        self.advance();
        if !self.eat(&Token::Semicolon) {
            while !self.at_end() && !self.at_named_end(&name) {
                self.advance();
            }
            if self.at_named_end(&name) {
                self.advance();
                self.advance();
                self.eat(&Token::Semicolon);
            }
        }
        let written = self.span_from(start);
        let header = self
            .source_text(written)
            .lines()
            .next()
            .unwrap_or_default()
            .to_owned();
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "Ada task declarations are not FPAS syntax",
            ForeignHabit {
                found: &header,
                replacement: "procedure Work(); … go Work() inside scope … end scope;",
                explanation: "FPAS has no task declarations: write the task's work as a routine and start it with `go`; `Std.Tasks` provides channels and `Wait`.",
            },
            written,
            None,
        );
    }

    fn at_named_end(&self, name: &str) -> bool {
        self.check(&Token::End)
            && matches!(self.peek_token(), Token::Ident(word) if word.eq_ignore_ascii_case(name))
    }
}
