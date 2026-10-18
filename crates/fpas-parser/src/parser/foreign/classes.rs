//! Delphi `class` types.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/types/record-methods.md`

use super::super::{ERROR_IDENT, Parser};
use crate::ast::{QualifiedId, TypeBody, TypeExpr};
use fpas_diagnostics::ForeignHabit;
use fpas_diagnostics::codes::PARSE_FOREIGN_SYNTAX;
use fpas_lexer::Token;

impl Parser {
    /// True at a Delphi `class` type body; `class` is not an FPAS keyword, so `type T = Class;`
    /// stays an alias of a user type named `Class`.
    pub(in crate::parser) fn at_delphi_class(&self) -> bool {
        matches!(self.current_token(), Token::Ident(name) if name.eq_ignore_ascii_case("class"))
            && !matches!(self.peek_token(), Token::Dot | Token::Of | Token::Semicolon)
    }

    /// Skips a Delphi class body through its `end`, leaving the terminator, and reports it once.
    pub(in crate::parser) fn skip_delphi_class(&mut self, type_name: &str) -> TypeBody {
        let start = self.advance().span;
        let mut depth = 0_usize;
        while !self.at_end() {
            match self.current_token() {
                Token::Record | Token::Case => depth += 1,
                Token::End if depth == 0 => {
                    self.advance();
                    break;
                }
                Token::End => depth -= 1,
                _ => {}
            }
            self.advance();
        }
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        let replacement = format!(
            "type {type_name} = record … function Name(Self: {type_name}): T; … end record;"
        );
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "Delphi `class` is not FPAS syntax",
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation: "FPAS has no classes or inheritance; declare a record whose methods take `Self` and return updated values.",
            },
            written,
            None,
        );
        TypeBody::Alias(TypeExpr::Named {
            id: QualifiedId {
                parts: vec![ERROR_IDENT.to_owned()],
                span: written,
            },
            span: written,
        })
    }
}
