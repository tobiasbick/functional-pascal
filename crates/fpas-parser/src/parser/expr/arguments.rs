//! Positional value arguments and explicitly marked caller storage.

use super::super::Parser;
use crate::ast::Expr;
use fpas_lexer::Token;

impl Parser {
    /// Parse comma-separated value expressions and explicit caller-storage paths.
    pub(crate) fn parse_arg_list(&mut self) -> Vec<Expr> {
        let mut args = vec![self.parse_argument()];
        while self.eat(&Token::Comma) {
            args.push(self.parse_argument());
        }
        args
    }

    fn parse_argument(&mut self) -> Expr {
        let start = self.current_span();
        if self.eat(&Token::Var) {
            let target = self.parse_designator();
            Expr::VarArgument(target, self.span_from(start))
        } else {
            self.parse_expression()
        }
    }
}
