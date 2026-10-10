use super::super::Parser;
use crate::ast::*;
use crate::error::ParseError;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    /// Parses one editor type fragment with the grammar in `docs/specs/grammar.ebnf`.
    pub(crate) fn parse_standalone_type(mut self) -> (TypeExpr, Vec<ParseError>) {
        let type_expr = self.parse_type_expr();
        if !self.at_end() {
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                &format!(
                    "Expected end of type expression, found `{}`",
                    super::super::token_display(self.current_token())
                ),
                "Remove trailing tokens so the fragment contains exactly one type expression.",
                self.current_span(),
            );
        }
        (type_expr, self.errors)
    }

    /// Parses a type expression using built-in and record application forms.
    /// See `docs/pascal/language/types/generics.md`.
    pub(crate) fn parse_type_expr(&mut self) -> TypeExpr {
        self.with_nesting(Self::parse_type_expr_inner)
    }

    fn parse_type_expr_inner(&mut self) -> TypeExpr {
        if matches!(
            self.current_token(),
            Token::Array
                | Token::Channel
                | Token::Task
                | Token::Result
                | Token::OptionKw
                | Token::Dict
        ) && self.peek_token() == &Token::Less
        {
            let span = self.current_span();
            let name = super::super::token_display(self.current_token()).into_owned();
            self.advance();
            let id = QualifiedId {
                parts: vec![name],
                span,
            };
            self.reject_angle_type_arguments(&id);
            return TypeExpr::Named {
                id,
                span: self.span_from(span),
            };
        }
        match self.current_token() {
            Token::Array => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::Of);
                let inner = self.parse_type_expr();
                TypeExpr::Array(Box::new(inner), self.span_from(start))
            }
            Token::Channel => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::Of);
                let inner = self.parse_type_expr();
                TypeExpr::Channel(Box::new(inner), self.span_from(start))
            }
            Token::Task => {
                let start = self.current_span();
                self.advance();
                if self.eat(&Token::Of) {
                    let inner = self.parse_type_expr();
                    return TypeExpr::Task(Box::new(inner), self.span_from(start));
                }
                // A bare `task` infers its result type from the initializer's spawned call.
                TypeExpr::Named {
                    id: QualifiedId {
                        parts: vec!["task".to_owned()],
                        span: start,
                    },
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
                let (ok_type, err_type) = self.parse_result_type_arguments();
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
                let inner_type = self.parse_type_expr();
                TypeExpr::Option {
                    inner_type: Box::new(inner_type),
                    span: self.span_from(start),
                }
            }
            Token::Dict => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::Of);
                let key_type = self.parse_type_expr();
                self.expect(&Token::To);
                let value_type = self.parse_type_expr();
                TypeExpr::Dict {
                    key_type: Box::new(key_type),
                    value_type: Box::new(value_type),
                    span: self.span_from(start),
                }
            }
            Token::Distinct => {
                self.error_with_code(
                    PARSE_EXPECTED_TOKEN,
                    "`distinct` is only valid in a type declaration",
                    "Declare a named type such as `type UserId = distinct integer;` and use `UserId` here.",
                    self.current_span(),
                );
                self.advance();
                self.parse_type_expr()
            }
            _ => self.parse_named_type_expr(),
        }
    }

    fn parse_named_type_expr(&mut self) -> TypeExpr {
        let start = self.current_span();
        let qid = self.parse_qualified_id();
        if self.check(&Token::Less) {
            self.reject_angle_type_arguments(&qid);
        }
        if self.eat(&Token::Of) {
            let arguments = self.parse_named_type_arguments();
            return TypeExpr::Application {
                id: qid,
                arguments,
                span: self.span_from(start),
            };
        }
        TypeExpr::Named {
            id: qid,
            span: self.span_from(start),
        }
    }
}
