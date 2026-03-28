use super::super::Parser;
use crate::ast::*;
use fpas_lexer::Token;

impl Parser {
    pub(super) fn parse_designator_or_call(&mut self) -> Expr {
        let start = self.current_span();
        let designator = self.parse_designator();

        if self.check(&Token::LParen) {
            self.advance();
            let args = if self.check(&Token::RParen) {
                Vec::new()
            } else {
                self.parse_arg_list()
            };
            self.expect(&Token::RParen);
            Expr::Call {
                designator,
                args,
                span: self.span_from(start),
            }
        } else {
            Expr::Designator(designator)
        }
    }
}
