use super::super::Parser;
use crate::ast::*;
use crate::parser::foreign::{BindingContext, BindingKeyword};
use fpas_diagnostics::codes::PARSE_INVALID_CALL_OR_ASSIGNMENT_FORM;
use fpas_lexer::Token;

impl Parser {
    /// Parses a scoped block while preserving an enclosing block ending.
    pub(super) fn parse_block(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance();
        let stmts = self.parse_statement_list();
        if self.at_enclosing_block_end() {
            self.error_with_code(
                fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN,
                "Expected `end;` before the enclosing block ending",
                "Close this scoped `begin` block with `end;` before the named block ending.",
                self.current_span(),
            );
        } else {
            self.expect(&Token::End);
        }
        Stmt::Block(stmts, self.span_from(start))
    }

    /// Parses an inline constant with an optional type annotation.
    pub(super) fn parse_const_stmt(&mut self) -> Stmt {
        self.parse_local_binding(BindingKeyword::Const)
    }

    /// Parses a reassignable inline binding.
    pub(super) fn parse_var_stmt(&mut self) -> Stmt {
        self.parse_local_binding(BindingKeyword::Var)
    }

    /// Parses one local binding; grouped names keep only the first after their diagnostic.
    fn parse_local_binding(&mut self, keyword: BindingKeyword) -> Stmt {
        let start = self.advance().span;
        let item = self.parse_binding_item(keyword, false);
        self.report_binding_habits(
            start,
            std::slice::from_ref(&item),
            BindingContext::Keyword,
            false,
        );
        let span = self.span_from(start);
        Self::binding_statements(item)
            .into_iter()
            .next()
            .unwrap_or(Stmt::Block(Vec::new(), span))
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
        } else {
            let base = if self.eat(&Token::LParen) {
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
