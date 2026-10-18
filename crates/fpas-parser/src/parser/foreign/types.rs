//! Ada type declarations `type T is …` and Pascal enumerations `T = (A, B, C)`, including the
//! Ada array form `array (1 .. N) of T`.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/types/enums.md`

use super::super::{ERROR_IDENT, Parser};
use crate::ast::TypeBody;
use fpas_diagnostics::codes::PARSE_FOREIGN_SYNTAX;
use fpas_diagnostics::{Applicability, Fix, ForeignHabit, SourceSpan, fix_title};
use fpas_lexer::{Span, Token};

impl Parser {
    /// True at a Pascal or Ada enumeration literal list `(A, B, C)`.
    pub(in crate::parser) fn at_enum_literal_list(&self) -> bool {
        self.check(&Token::LParen)
            && matches!(self.peek_token(), Token::Ident(_))
            && matches!(
                self.tokens.get(self.pos + 2).map(|token| &token.token),
                Some(Token::Comma | Token::RParen)
            )
    }

    /// True at the Ada array type form `array (Low .. High) of T`.
    pub(in crate::parser) fn at_ada_array_type(&self) -> bool {
        self.check(&Token::Array) && self.peek_token() == &Token::LParen
    }

    /// Parses `(A, B, C)` and returns a placeholder body and the FPAS enum form.
    pub(in crate::parser) fn parse_enum_literal_list(&mut self, start: Span) -> (TypeBody, String) {
        self.advance();
        let mut members = Vec::new();
        while let Some((member, _)) = self.expect_ident() {
            members.push(member);
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::RParen);
        let form = format!("enum {}; end enum", members.join("; "));
        (self.placeholder_type_body(start), form)
    }

    /// Parses `array (Low .. High) of T` and returns a placeholder body and the FPAS form.
    pub(in crate::parser) fn parse_ada_array_type(&mut self, start: Span) -> (TypeBody, String) {
        self.advance();
        let mut depth = 0_usize;
        while !self.at_end() {
            match self.current_token() {
                Token::LParen => depth += 1,
                Token::RParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        self.advance();
                        break;
                    }
                }
                Token::Semicolon => break,
                _ => {}
            }
            self.advance();
        }
        self.expect(&Token::Of);
        let element_start = self.current_span();
        self.parse_type_expr();
        let element = self.source_text(self.span_from(element_start)).to_owned();
        (
            self.placeholder_type_body(start),
            format!("array of {element}"),
        )
    }

    /// Reports an Ada `type T is …` or a Pascal enumeration once, with the FPAS declaration.
    pub(in crate::parser) fn report_type_habit(
        &mut self,
        start: Span,
        name: &str,
        ada_is: bool,
        body_form: &str,
    ) {
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        let replacement = format!("type {name} = {body_form};");
        // Ada arrays may start at any index; the FPAS array is zero-based.
        let fix = if body_form.contains('…') || body_form.contains("TypeName") {
            None
        } else if ada_is && body_form.starts_with("array of") {
            Some(Applicability::NeedsReview)
        } else {
            Some(Applicability::MachineApplicable)
        };
        let (message, explanation) = if ada_is {
            (
                "Ada type declaration `type Name is …` is not FPAS syntax",
                "Type declarations use `=`; enumerations are `enum … end enum` and arrays have no bounds.",
            )
        } else {
            (
                "Pascal enumeration `(A, B, C)` is not FPAS syntax",
                "Enumerations are declared with `enum`, one member per `;`.",
            )
        };
        let index = self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            message,
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation,
            },
            written,
            None,
        );
        // The written span starts at the name, after `type`, and ends before `;`; type
        // parameters between the name and `=` or `is` rule the fix out.
        let after_name = found.get(name.len()..).unwrap_or_default().trim_start();
        let plain_name = after_name.starts_with('=')
            || after_name
                .get(..2)
                .is_some_and(|word| word.eq_ignore_ascii_case("is"));
        let fix = fix.filter(|_| plain_name).and_then(|applicability| {
            Fix::replace(
                fix_title(&replacement),
                applicability,
                SourceSpan::try_from(written).ok()?,
                format!("{name} = {body_form}"),
            )
        });
        self.add_habit_fix(index, fix);
    }

    fn placeholder_type_body(&self, span: Span) -> TypeBody {
        TypeBody::Alias(Self::named_type(ERROR_IDENT, span))
    }
}
