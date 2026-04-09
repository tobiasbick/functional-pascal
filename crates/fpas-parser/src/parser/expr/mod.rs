mod postfix;
mod precedence;
mod primary;

use super::Parser;
use crate::ast::*;
use fpas_lexer::Token;

impl Parser {
    pub(crate) fn parse_expression(&mut self) -> Expr {
        self.parse_comparison()
    }

    pub(crate) fn parse_designator(&mut self) -> Designator {
        let start = self.current_span();
        let mut parts = Vec::new();

        let (name, name_span) = if let Some(p) = self.try_consume_std_keyword_path_segment() {
            p
        } else {
            self.expect_ident().unwrap_or(("_error_".into(), start))
        };
        parts.push(DesignatorPart::Ident(name, name_span));

        loop {
            if self.eat(&Token::Dot) {
                let (name, name_span) = self
                    .expect_ident_after_dot()
                    .unwrap_or(("_error_".into(), self.current_span()));
                parts.push(DesignatorPart::Ident(name, name_span));
            } else if self.eat(&Token::LBracket) {
                let idx_start = self.current_span();
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
