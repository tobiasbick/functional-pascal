//! Delphi and C-family type applications in angle brackets, such as `TList<integer>`.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/types/generics.md`

use super::super::Parser;
use crate::QualifiedId;
use fpas_diagnostics::ForeignHabit;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    /// Rejects a type application in angle brackets while recovering its arguments.
    pub(in crate::parser) fn reject_angle_type_arguments(&mut self, name: &QualifiedId) {
        self.advance();
        let mut arguments = Vec::new();
        if !self.check(&Token::Greater) {
            loop {
                let start = self.current_span();
                self.parse_type_expr();
                arguments.push(self.source_text(self.span_from(start)).to_owned());
                if !self.eat(&Token::Comma) {
                    break;
                }
            }
        }
        self.expect(&Token::Greater);
        let written = self.span_from(name.span);
        let found = self.source_text(written).to_owned();
        let type_name = name.parts.join(".");
        let argument = |index: usize, fallback: &str| {
            arguments
                .get(index)
                .cloned()
                .unwrap_or_else(|| fallback.to_owned())
        };
        let replacement = match type_name.to_ascii_lowercase().as_str() {
            "result" => format!("Result of ({}, {})", argument(0, "T"), argument(1, "E")),
            "dict" | "tdictionary" => {
                format!("dict of {} to {}", argument(0, "K"), argument(1, "V"))
            }
            "tlist" | "tobjectlist" | "array" => format!("array of {}", argument(0, "T")),
            _ if arguments.len() > 1 => format!("{type_name} of ({})", arguments.join(", ")),
            _ => format!("{type_name} of {}", argument(0, "T")),
        };
        self.report_foreign_habit(
            PARSE_EXPECTED_TOKEN,
            "Type applications use `of`, not angle brackets",
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation: "Generic types take their arguments after `of`; only routine declarations use `<T>` parameters.",
            },
            written,
        );
    }
}
