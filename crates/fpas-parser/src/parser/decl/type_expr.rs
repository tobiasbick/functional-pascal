//! Canonical parenthesized type applications and callable annotations.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use super::super::Parser;
use crate::ast::*;
use fpas_lexer::Token;

impl Parser {
    /// Parse one type annotation within the shared nesting budget.
    pub(crate) fn parse_type_expr(&mut self) -> TypeExpr {
        self.with_nesting(Self::parse_type_expr_inner)
    }

    fn parse_type_expr_inner(&mut self) -> TypeExpr {
        match self.current_token() {
            Token::Array => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::Of);
                self.expect_type_argument_list();
                let inner = self.parse_type_expr();
                self.expect(&Token::RParen);
                TypeExpr::Array(Box::new(inner), self.span_from(start))
            }
            Token::Channel => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::Of);
                self.expect_type_argument_list();
                let inner = self.parse_type_expr();
                self.expect(&Token::RParen);
                TypeExpr::Channel(Box::new(inner), self.span_from(start))
            }
            Token::Task => {
                let start = self.current_span();
                self.advance();
                if self.eat(&Token::Of) {
                    self.expect_type_argument_list();
                    let inner = self.parse_type_expr();
                    self.expect(&Token::RParen);
                    return TypeExpr::Task(Box::new(inner), self.span_from(start));
                }
                // A bare `task` infers its result type from the initializer's spawned call.
                TypeExpr::Named {
                    id: QualifiedId {
                        parts: vec!["task".to_owned()],
                        span: start,
                    },
                    arguments: Vec::new(),
                    span: start,
                }
            }
            Token::Function => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::LParen);
                let params = self.parse_formal_param_list(false);
                self.expect(&Token::RParen);
                self.expect(&Token::Colon);
                let return_type = self.parse_type_expr();
                TypeExpr::FunctionType {
                    params,
                    return_type: Box::new(return_type),
                    span: self.span_from(start),
                }
            }
            Token::Procedure => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::LParen);
                let params = self.parse_formal_param_list(false);
                self.expect(&Token::RParen);
                TypeExpr::ProcedureType {
                    params,
                    span: self.span_from(start),
                }
            }
            Token::Result => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::Of);
                self.expect_type_argument_list();
                let ok_type = self.parse_type_expr();
                self.expect(&Token::Comma);
                let err_type = self.parse_type_expr();
                self.expect(&Token::RParen);
                TypeExpr::Result {
                    ok_type: Box::new(ok_type),
                    err_type: Box::new(err_type),
                    span: self.span_from(start),
                }
            }
            Token::OptionKw => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::Of);
                self.expect_type_argument_list();
                let inner_type = self.parse_type_expr();
                self.expect(&Token::RParen);
                TypeExpr::Option {
                    inner_type: Box::new(inner_type),
                    span: self.span_from(start),
                }
            }
            Token::Dict => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::Of);
                self.expect_type_argument_list();
                let key_type = self.parse_type_expr();
                self.expect(&Token::Comma);
                let value_type = self.parse_type_expr();
                self.expect(&Token::RParen);
                TypeExpr::Dict {
                    key_type: Box::new(key_type),
                    value_type: Box::new(value_type),
                    span: self.span_from(start),
                }
            }
            _ => self.parse_named_type_expr(),
        }
    }

    fn expect_type_argument_list(&mut self) {
        if !self.eat(&Token::LParen) {
            self.error_with_code(
                fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN,
                "Type arguments after `of` require parentheses",
                "Write `array of (integer)`, `Result of (integer, string)`, or `dict of (string, integer)`.",
                self.current_span(),
            );
        }
    }

    fn parse_named_type_expr(&mut self) -> TypeExpr {
        let start = self.current_span();
        let qid = self.parse_qualified_id();
        let mut arguments = Vec::new();
        if self.eat(&Token::Of) {
            self.expect_type_argument_list();
            arguments.push(self.parse_type_expr());
            while self.eat(&Token::Comma) {
                arguments.push(self.parse_type_expr());
            }
            self.expect(&Token::RParen);
        }
        TypeExpr::Named {
            id: qid,
            arguments,
            span: self.span_from(start),
        }
    }
}
