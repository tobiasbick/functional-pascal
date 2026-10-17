//! Delphi and Ada habits in routine headers, parameters and results: `const` parameters,
//! headers without a parameter list, Ada `function F return T is … end F;` and Delphi
//! `Result :=`.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/functions/declarations.md`

use super::super::Parser;
use crate::ast::{FuncBody, Stmt};
use fpas_diagnostics::ForeignHabit;
use fpas_diagnostics::codes::{PARSE_EXPECTED_TOKEN, PARSE_FOREIGN_SYNTAX};
use fpas_lexer::{Span, Token};

/// Source text of a routine header, collected to build the FPAS form of an Ada header.
pub(in crate::parser) struct AdaHeaderParts<'a> {
    /// `function` or `procedure`.
    pub keyword: &'a str,
    /// Routine name as written.
    pub name: &'a str,
    /// Parameter list including parentheses, or `None` when it was omitted.
    pub params: Option<Span>,
    /// Result type of a function.
    pub return_type: Option<Span>,
}

impl Parser {
    /// Consumes a Delphi `const` parameter modifier and returns its span.
    pub(in crate::parser) fn eat_const_parameter_modifier(&mut self) -> Option<Span> {
        if self.check(&Token::Const) && matches!(self.peek_token(), Token::Ident(_)) {
            Some(self.advance().span)
        } else {
            None
        }
    }

    /// Reports a `const` parameter modifier once the parameter after it is parsed.
    pub(in crate::parser) fn report_const_parameter(&mut self, modifier: Span, name: Span) {
        let written = self.span_from(modifier);
        let found = self.source_text(written).to_owned();
        let replacement = self.source_text(self.span_from(name)).to_owned();
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "Delphi `const` parameter modifier is not FPAS syntax",
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation: "Parameters are read-only already; remove `const`.",
            },
            written,
        );
    }

    /// True when a routine header continues without its parameter list.
    pub(in crate::parser) fn at_missing_parameter_list(&self) -> bool {
        matches!(
            self.current_token(),
            Token::Semicolon | Token::Colon | Token::Return | Token::Is
        )
    }

    /// Reports `procedure Q;` or `function F: T;`; an Ada header reports instead.
    pub(in crate::parser) fn report_missing_parameter_list(&mut self, name: &str, name_span: Span) {
        if matches!(self.current_token(), Token::Return | Token::Is) {
            return;
        }
        let replacement = format!("{name}()");
        self.report_foreign_habit(
            PARSE_EXPECTED_TOKEN,
            &format!("Routine `{name}` has no parameter list"),
            ForeignHabit {
                found: name,
                replacement: &replacement,
                explanation: "Routines always declare a parameter list, also an empty one.",
            },
            name_span,
        );
    }

    /// Reports an Ada routine header written from `start` up to the current token.
    pub(in crate::parser) fn report_ada_routine_header(
        &mut self,
        start: Span,
        parts: AdaHeaderParts<'_>,
    ) {
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        let params = parts
            .params
            .map_or("()", |span| self.source_text(span))
            .to_owned();
        let replacement = match parts.return_type {
            Some(span) => format!(
                "{} {}{params}: {};",
                parts.keyword,
                parts.name,
                self.source_text(span)
            ),
            None => format!("{} {}{params};", parts.keyword, parts.name),
        };
        let explanation = if parts.return_type.is_some() {
            "FPAS declares the result type after `:`, has no `is`, and closes the body with `end function;`."
        } else {
            "FPAS has no `is` in routine headers and closes the body with `end procedure;`."
        };
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "Ada routine header is not FPAS syntax",
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation,
            },
            written,
        );
    }

    /// Skips an Ada routine body through `end Name;` (or a named FPAS ending) without
    /// reporting its contents, and stops before the program's `end.`.
    pub(in crate::parser) fn skip_ada_routine_body(&mut self, name: &str) -> FuncBody {
        while !self.at_end() {
            if self.check(&Token::End) {
                match self.peek_token() {
                    Token::Ident(ending) if ending.eq_ignore_ascii_case(name) => {
                        self.advance();
                        self.advance();
                        self.eat(&Token::Semicolon);
                        break;
                    }
                    Token::Function | Token::Procedure => {
                        self.advance();
                        self.advance();
                        self.eat(&Token::Semicolon);
                        break;
                    }
                    Token::Dot => break,
                    _ => {}
                }
            }
            self.advance();
        }
        FuncBody::Block {
            nested: Vec::new(),
            stmts: Vec::new(),
        }
    }

    /// True at Delphi `Result :=` or `Result.Field :=`; `Result` is otherwise only a type keyword.
    pub(in crate::parser) fn at_delphi_result_assignment(&self) -> bool {
        self.check(&Token::Result) && matches!(self.peek_token(), Token::ColonAssign | Token::Dot)
    }

    /// Parses Delphi `Result := Value` as `return Value`, reporting the habit once.
    pub(in crate::parser) fn parse_delphi_result_assignment(&mut self) -> Stmt {
        let start = self.advance().span;
        let mut fields = Vec::new();
        while self.eat(&Token::Dot) {
            match self.expect_ident() {
                Some((field, _)) => fields.push(field),
                None => break,
            }
        }
        self.expect(&Token::ColonAssign);
        let value = self.parse_expression();
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        let value_text = self.source_text(value.span()).to_owned();
        let (replacement, explanation) = match fields.first() {
            None => (
                format!("return {value_text}"),
                "FPAS has no implicit `Result` variable; a function returns its value with `return`.",
            ),
            Some(field) => (
                format!("return TypeName({field} := {value_text}, …)"),
                "FPAS has no implicit `Result` variable; build the complete record and return it.",
            ),
        };
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "Delphi `Result :=` does not set a function result",
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation,
            },
            written,
        );
        if fields.is_empty() {
            Stmt::Return(Some(value), written)
        } else {
            Stmt::Block(Vec::new(), written)
        }
    }
}
