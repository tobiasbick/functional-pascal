//! Anonymous function and procedure expressions (capturing closures).
//!
//! **Documentation:** `docs/pascal/language/functions/closures.md`

use super::super::Parser;
use crate::ast::*;
use fpas_lexer::Token;

impl Parser {
    /// Parse `function(…) : T begin … end` or `procedure(…) begin … end` as an expression.
    ///
    /// The keyword has not been consumed yet. Distinguishes from named declarations by
    /// requiring `(` immediately after `function` / `procedure`.
    pub(super) fn parse_closure_expr(&mut self) -> Expr {
        let start = self.current_span();
        let is_function = self.check(&Token::Function);
        self.advance(); // function | procedure

        self.expect(&Token::LParen);
        let params = self.parse_formal_param_list();
        self.expect(&Token::RParen);

        let return_type = if is_function {
            self.expect(&Token::Colon);
            Some(self.parse_type_expr())
        } else {
            None
        };

        let body = self.parse_closure_body();
        Expr::Closure(Box::new(ClosureExpr {
            is_function,
            params,
            return_type,
            body,
            span: self.span_from(start),
        }))
    }

    /// Closure body ends with `end` and does not consume a trailing semicolon.
    fn parse_closure_body(&mut self) -> FuncBody {
        let nested = self.parse_nested_decls();
        self.expect(&Token::Begin);
        let stmts = self.parse_statement_list();
        self.expect(&Token::End);
        FuncBody::Block { nested, stmts }
    }

    /// True when the current token starts an anonymous routine expression.
    pub(in crate::parser) fn at_closure_expr_start(&self) -> bool {
        matches!(self.current_token(), Token::Function | Token::Procedure)
            && self.peek_token() == &Token::LParen
    }
}
