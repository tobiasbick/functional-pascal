mod basic;
mod case;
mod concurrency;
mod conditionals;
mod control_bodies;
mod loops;
mod terminators;

use super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::PARSE_INVALID_STATEMENT_START;
use fpas_lexer::Token;

impl Parser {
    fn parse_statement(&mut self) -> Stmt {
        self.with_nesting(Self::parse_statement_inner)
    }

    fn parse_statement_inner(&mut self) -> Stmt {
        match self.current_token() {
            Token::Begin => self.parse_block(),
            Token::Null | Token::Discard if self.peek_token() == &Token::ColonAssign => {
                self.parse_invalid_statement_start()
            }
            Token::Null => Stmt::Null(self.advance().span),
            Token::Var => self.parse_var_stmt(false),
            Token::Mutable if self.is_mutable_var_start() => self.parse_var_stmt(true),
            Token::Mutable => self.parse_invalid_statement_start(),
            Token::Return => self.parse_return_stmt(),
            Token::Discard => {
                let start = self.advance().span;
                let expr = self.parse_expression();
                Stmt::Discard {
                    expr,
                    span: self.span_from(start),
                }
            }
            Token::Panic => self.parse_panic_stmt(),
            Token::If => self.parse_if_stmt(),
            Token::Case => self.parse_case_stmt(),
            Token::For => self.parse_for_stmt(),
            Token::While => self.parse_while_stmt(),
            Token::Repeat => self.parse_repeat_stmt(),
            Token::Break => {
                let span = self.current_span();
                self.advance();
                Stmt::Break(span)
            }
            Token::Continue => {
                let span = self.current_span();
                self.advance();
                Stmt::Continue(span)
            }
            Token::Go => self.parse_go_stmt(),
            Token::Ident(_) | Token::SelfKw => self.parse_call_or_assign(),
            _ if self.can_start_expression() => self.parse_expression_stmt(),
            _ => self.parse_invalid_statement_start(),
        }
    }

    fn can_start_statement(&self) -> bool {
        matches!(
            self.current_token(),
            Token::Begin
                | Token::Null
                | Token::Var
                | Token::Mutable
                | Token::Return
                | Token::Discard
                | Token::Panic
                | Token::If
                | Token::Case
                | Token::For
                | Token::While
                | Token::Repeat
                | Token::Break
                | Token::Continue
        ) || self.can_start_expression()
    }

    fn parse_invalid_statement_start(&mut self) -> Stmt {
        let span = self.current_span();
        self.error_with_code(
            PARSE_INVALID_STATEMENT_START,
            &format!(
                "Unexpected token `{}` at start of statement",
                super::token_display(self.current_token())
            ),
            self.reserved_identifier_hint()
                .unwrap_or("Expected a statement: var, if, while, for, begin, return, etc."),
            span,
        );
        while !self.at_end() && !self.check(&Token::Semicolon) && !self.is_stmt_list_end() {
            self.advance();
        }
        Stmt::Block(Vec::new(), span)
    }

    pub(in crate::parser) fn parse_go_call_expression(
        &mut self,
        go_span: fpas_lexer::Span,
    ) -> Expr {
        let expr = self.parse_expression();
        if matches!(expr, Expr::Call { .. })
            || matches!(&expr, Expr::Postfix { operations, .. }
                if matches!(operations.last(), Some(crate::ast::PostfixOperation::MethodCall { .. })))
        {
            expr
        } else {
            self.error_with_code(
                fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION,
                "`go` requires a function or procedure call expression",
                "Use `go FunctionName(args)` or `go SomeCallable(args)`.",
                go_span,
            );
            Expr::Error(expr.span())
        }
    }

    fn can_start_expression(&self) -> bool {
        self.is_ident_designator_start()
            || self.at_closure_expr_start()
            || matches!(
                self.current_token(),
                Token::Integer(_)
                    | Token::Real(_)
                    | Token::Str(_)
                    | Token::True
                    | Token::False
                    | Token::LParen
                    | Token::LBracket
                    | Token::Not
                    | Token::Minus
                    | Token::Record
                    | Token::Ok
                    | Token::Error
                    | Token::Some
                    | Token::None
                    | Token::Nil
                    | Token::Try
                    | Token::Go
            )
    }
}
