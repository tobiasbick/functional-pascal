mod basic;
mod branching;
mod concurrency;
mod loops;

use super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::PARSE_INVALID_STATEMENT_START;
use fpas_lexer::Token;

impl Parser {
    pub(crate) fn parse_statement_list(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();

        if self.is_stmt_list_end() {
            return stmts;
        }

        stmts.push(self.parse_statement());
        while self.eat(&Token::Semicolon) {
            if self.is_stmt_list_end() {
                break;
            }
            stmts.push(self.parse_statement());
        }
        stmts
    }

    fn is_stmt_list_end(&self) -> bool {
        matches!(
            self.current_token(),
            Token::End | Token::Else | Token::Until | Token::Eof
        )
    }

    fn parse_statement(&mut self) -> Stmt {
        match self.current_token() {
            Token::Begin => self.parse_block(),
            Token::Var => self.parse_var_stmt(false),
            Token::Mutable => {
                if self.peek_token() == &Token::Var {
                    self.parse_var_stmt(true)
                } else {
                    self.parse_invalid_statement_start()
                }
            }
            Token::Return => self.parse_return_stmt(),
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
            Token::Ident(_) => self.parse_call_or_assign(),
            _ => self.parse_invalid_statement_start(),
        }
    }

    fn parse_invalid_statement_start(&mut self) -> Stmt {
        let span = self.current_span();
        self.error_with_code(
            PARSE_INVALID_STATEMENT_START,
            &format!(
                "Unexpected token `{}` at start of statement",
                super::token_display(self.current_token())
            ),
            "Expected a statement: var, if, while, for, begin, return, etc.",
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
        if matches!(expr, Expr::Call { .. }) {
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
        self.is_std_keyword_path_start()
            || matches!(
                self.current_token(),
                Token::Integer(_)
                    | Token::Real(_)
                    | Token::Str(_)
                    | Token::True
                    | Token::False
                    | Token::Ident(_)
                    | Token::LParen
                    | Token::LBracket
                    | Token::Not
                    | Token::Minus
                    | Token::Record
                    | Token::Ok
                    | Token::Error
                    | Token::Some
                    | Token::None
                    | Token::Try
                    | Token::Go
                    | Token::Function
            )
    }
}
