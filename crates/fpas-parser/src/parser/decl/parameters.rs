//! Formal parameters and recovery for non-canonical comma/group declarations.
//!
//! Documentation: `docs/pascal/language/functions/parameters.md` and
//! `docs/pascal/tools/diagnostics.md`.

use super::super::Parser;
use crate::ast::{FormalParam, TypeExpr};
use crate::error::parse_error;
use fpas_diagnostics::codes::{PARSE_EXPECTED_TOKEN, PARSE_INVALID_PARAMETER_SEPARATOR};
use fpas_lexer::Token;

impl Parser {
    pub(in crate::parser) fn parse_formal_param_list(
        &mut self,
        allow_self_receiver: bool,
    ) -> Vec<FormalParam> {
        let mut params = Vec::new();
        if self.check(&Token::RParen) {
            return params;
        }
        let Some(param) = self.parse_formal_param(allow_self_receiver) else {
            return params;
        };
        params.push(param);
        loop {
            if self.check(&Token::Comma) {
                self.invalid_parameter_separator(false);
                break;
            }
            if !self.eat(&Token::Semicolon) {
                break;
            }
            if self.check(&Token::RParen) {
                self.error_with_code(
                    PARSE_EXPECTED_TOKEN,
                    "Unexpected `;` before `)` in a parameter list",
                    "Remove the trailing semicolon, or add another parameter before `)`.",
                    self.current_span(),
                );
                break;
            }
            let Some(param) = self.parse_formal_param(false) else {
                break;
            };
            params.push(param);
        }
        params
    }

    fn parse_formal_param(&mut self, allow_self_receiver: bool) -> Option<FormalParam> {
        let start = self.current_span();
        if matches!(self.current_token(), Token::Ident(name) if name.eq_ignore_ascii_case("mutable"))
            && matches!(self.peek_token(), Token::Ident(_) | Token::SelfKw)
        {
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "The `mutable` parameter modifier has been removed",
                "Parameters are read-only. Write `Value: integer` and start the body with `var LocalValue: integer := Value;` when a writable local copy is needed.",
                start,
            );
            self.advance();
        }
        let (name, _) = if allow_self_receiver && self.check(&Token::SelfKw) {
            let span = self.advance().span;
            ("Self".to_owned(), span)
        } else {
            self.expect_ident()
                .unwrap_or_else(|| self.error_ident(start))
        };
        if self.check(&Token::Comma) {
            self.invalid_parameter_separator(true);
            return None;
        }
        self.expect(&Token::Colon);
        let type_expr: TypeExpr = self.parse_type_expr();
        Some(FormalParam {
            name,
            type_expr,
            span: self.span_from(start),
        })
    }

    fn invalid_parameter_separator(&mut self, grouped: bool) {
        let message = if grouped {
            "Grouped parameter names are invalid; each parameter needs its own type"
        } else {
            "Parameter declarations must be separated by `;`, not `,`"
        };
        let diagnostic = parse_error(
            PARSE_INVALID_PARAMETER_SEPARATOR,
            message,
            "Declare each parameter with its own type and separate parameters with `;`, for example `function Add(A: integer; B: integer): integer;` or `procedure Print(A: integer; B: integer);`. For anonymous routines, omit the name.",
            self.current_span(),
        )
        .with_expected_found("individually typed parameters separated by ;", ",");
        if !self.nesting_limit_reached {
            self.errors.push(diagnostic);
        }
        // Leave the enclosing `)` for the header parser; nested callable types
        // can have their own parentheses inside the rejected list.
        let mut depth = 0_usize;
        while !self.at_end() {
            match self.current_token() {
                Token::RParen if depth == 0 => break,
                Token::RParen => depth -= 1,
                Token::LParen => depth += 1,
                Token::Begin | Token::End if depth == 0 => break,
                _ => {}
            }
            self.advance();
        }
    }
}
