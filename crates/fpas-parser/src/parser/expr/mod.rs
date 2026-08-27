mod closure;
mod postfix;
mod precedence;
mod primary;

use super::Parser;
use crate::ast::*;
use crate::error::ParseError;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    pub(crate) fn parse_standalone_expression(mut self) -> (Expr, Vec<ParseError>) {
        let expression = self.parse_expression();
        if !self.at_end() {
            let span = self.current_span();
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                &format!(
                    "Expected end of expression, found `{}`",
                    super::token_display(self.current_token())
                ),
                "Remove trailing tokens so the debugger receives exactly one expression.",
                span,
            );
            while !self.at_end() {
                self.advance();
            }
        }
        (expression, self.errors)
    }

    pub(crate) fn parse_expression(&mut self) -> Expr {
        self.with_nesting(Self::parse_comparison)
    }

    pub(crate) fn parse_designator(&mut self) -> Designator {
        let start = self.current_span();
        let mut parts = Vec::new();

        let (name, name_span) = if let Some(p) = self.try_consume_std_keyword_path_segment() {
            p
        } else {
            self.expect_ident()
                .unwrap_or_else(|| self.error_ident(start))
        };
        parts.push(DesignatorPart::Ident(name, name_span));

        loop {
            if self.eat(&Token::Dot) {
                let (name, name_span) = self
                    .expect_ident_after_dot()
                    .unwrap_or_else(|| self.error_ident(self.current_span()));
                parts.push(DesignatorPart::Ident(name, name_span));
            } else if self.check(&Token::LBracket) {
                let idx_start = self.current_span();
                self.advance();
                let index = self.parse_expression();
                self.expect(&Token::RBracket);
                parts.push(DesignatorPart::Index(index, self.span_from(idx_start)));
            } else {
                break;
            }
        }

        Designator {
            parts,
            span: self.span_from(start),
        }
    }

    pub(crate) fn parse_arg_list(&mut self) -> Vec<Expr> {
        let mut args = Vec::new();
        args.push(self.parse_expression());
        while self.eat(&Token::Comma) {
            args.push(self.parse_expression());
        }
        args
    }
}
