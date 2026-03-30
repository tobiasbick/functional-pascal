use super::super::Parser;
use crate::ast::*;
use fpas_lexer::Token;

impl Parser {
    pub(super) fn parse_block(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance();
        let stmts = self.parse_statement_list();
        self.expect(&Token::End);
        Stmt::Block(stmts, self.span_from(start))
    }

    pub(super) fn parse_var_stmt(&mut self, mutable: bool) -> Stmt {
        let start = self.current_span();
        if mutable {
            self.advance();
        }
        self.advance();
        let (name, _) = match self.expect_ident() {
            Some(ident) => ident,
            None => {
                if !self.at_end() {
                    self.advance();
                }
                ("_error_".into(), start)
            }
        };
        self.expect(&Token::Colon);
        let type_expr = self.parse_type_expr();
        self.expect(&Token::ColonAssign);
        let value = self.parse_expression();
        let var_def = VarDef {
            name,
            type_expr,
            value,
            visibility: Visibility::default(),
            span: self.span_from(start),
        };
        if mutable {
            Stmt::MutableVar(var_def)
        } else {
            Stmt::Var(var_def)
        }
    }

    pub(super) fn parse_return_stmt(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance();

        let expr = if self.can_start_expression() {
            Some(self.parse_expression())
        } else {
            None
        };

        Stmt::Return(expr, self.span_from(start))
    }

    pub(super) fn parse_panic_stmt(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance();
        self.expect(&Token::LParen);
        let expr = self.parse_expression();
        self.expect(&Token::RParen);
        Stmt::Panic(expr, self.span_from(start))
    }

    pub(super) fn parse_call_or_assign(&mut self) -> Stmt {
        let start = self.current_span();
        let designator = self.parse_designator();

        if self.eat(&Token::ColonAssign) {
            let value = self.parse_expression();
            Stmt::Assign {
                target: designator,
                value,
                span: self.span_from(start),
            }
        } else if self.check(&Token::LParen) {
            self.advance();
            let args = if self.check(&Token::RParen) {
                Vec::new()
            } else {
                self.parse_arg_list()
            };
            self.expect(&Token::RParen);
            Stmt::Call {
                designator,
                args,
                span: self.span_from(start),
            }
        } else {
            Stmt::Call {
                designator,
                args: Vec::new(),
                span: self.span_from(start),
            }
        }
    }
}
