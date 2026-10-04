use super::super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::PARSE_INVALID_CALL_OR_ASSIGNMENT_FORM;
use fpas_lexer::Token;

impl Parser {
    pub(super) fn parse_block(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance();
        let stmts = self.parse_statement_list();
        if self.check(&Token::End) && matches!(self.peek_token(), Token::Semicolon | Token::Eof) {
            self.advance();
        } else {
            self.error_with_code(
                fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN,
                "Expected bare `end` to close the plain lexical block",
                "Close the explicit `begin` block with `end;` before the enclosing named closer.",
                self.current_span(),
            );
        }
        Stmt::Block(stmts, self.span_from(start))
    }

    pub(super) fn parse_var_stmt(&mut self, mutable: bool) -> Stmt {
        let start = self.current_span();
        self.advance();
        let mut var_def = self.parse_binding_definition(true, mutable, Visibility::default());
        var_def.span = self.span_from(start);
        if mutable {
            Stmt::Var(var_def)
        } else {
            Stmt::Const(var_def)
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

    /// Parse an explicit value consumer; see `docs/pascal/language/functions/first-class.md`.
    pub(super) fn parse_discard_stmt(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance();
        let expr = self.parse_expression();
        Stmt::Discard(expr, self.span_from(start))
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
        } else {
            let base = if !designator
                .parts
                .iter()
                .any(|part| matches!(part, DesignatorPart::Index(..)))
                && self.eat(&Token::LParen)
            {
                if self.at_named_field() {
                    self.parse_record_construction(designator, start)
                } else {
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
                }
            } else {
                Expr::Designator(designator)
            };
            let expr = self.apply_postfix_suffixes(base, start);
            match expr {
                Expr::Call {
                    designator,
                    args,
                    span,
                } => Stmt::Call {
                    designator,
                    args,
                    span,
                },
                Expr::Designator(designator) => {
                    self.error_with_code(
                        PARSE_INVALID_CALL_OR_ASSIGNMENT_FORM,
                        "Expected `(` for a call or `:=` for an assignment after the designator",
                        "Add `()` to call a zero-argument function or procedure.",
                        designator.span,
                    );
                    Stmt::Call {
                        span: designator.span,
                        designator,
                        args: Vec::new(),
                    }
                }
                expr => Stmt::Expression {
                    span: self.span_from(start),
                    expr,
                },
            }
        }
    }

    pub(super) fn parse_expression_stmt(&mut self) -> Stmt {
        let start = self.current_span();
        let expr = self.parse_expression();
        Stmt::Expression {
            expr,
            span: self.span_from(start),
        }
    }
}
