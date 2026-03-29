use super::super::Parser;
use crate::ast::*;
use fpas_lexer::Token;

impl Parser {
    pub(crate) fn parse_type_expr(&mut self) -> TypeExpr {
        match self.current_token() {
            Token::Array => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::Of);
                let inner = self.parse_type_expr();
                TypeExpr::Array(Box::new(inner), self.span_from(start))
            }
            Token::Function => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::LParen);
                let params = self.parse_formal_param_list();
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
                let params = self.parse_formal_param_list();
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
                let ok_type = self.parse_type_expr();
                self.expect(&Token::Comma);
                let err_type = self.parse_type_expr();
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
            Token::Channel => {
                let start = self.current_span();
                self.advance();
                self.expect(&Token::Of);
                let inner_type = self.parse_type_expr();
                TypeExpr::Channel {
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
            _ => {
                let qid = self.parse_qualified_id();
                // Check for `of` type arguments: `Stack of integer`
                let type_args = if self.eat(&Token::Of) {
                    self.parse_type_arg_list()
                } else {
                    Vec::new()
                };
                TypeExpr::Named { id: qid, type_args }
            }
        }
    }

    /// Parse a comma-separated list of type arguments (after `of`):
    /// `integer` or `integer, string`.
    fn parse_type_arg_list(&mut self) -> Vec<TypeExpr> {
        let mut args = vec![self.parse_type_expr()];
        while self.eat(&Token::Comma) {
            args.push(self.parse_type_expr());
        }
        args
    }

    /// Parse optional generic type parameters: `<T>`, `<T: Comparable>`, `<A, B>`.
    /// Returns an empty vec if no `<` follows.
    ///
    /// **Documentation:** `docs/pascal/05-types.md` (Generics — Constraints)
    pub(crate) fn parse_type_params(&mut self) -> Vec<crate::TypeParam> {
        if !self.eat(&Token::Less) {
            return Vec::new();
        }
        let mut params = Vec::new();
        params.push(self.parse_single_type_param());
        while self.eat(&Token::Comma) {
            params.push(self.parse_single_type_param());
        }
        self.expect(&Token::Greater);
        params
    }

    /// Parse a single type parameter: `T` or `T: Constraint`.
    fn parse_single_type_param(&mut self) -> crate::TypeParam {
        let (name, _) = self
            .expect_ident()
            .unwrap_or(("_error_".into(), self.current_span()));
        let constraint = if self.eat(&Token::Colon) {
            let (constraint_name, _) = self
                .expect_ident()
                .unwrap_or(("_error_".into(), self.current_span()));
            Some(constraint_name)
        } else {
            None
        };
        crate::TypeParam { name, constraint }
    }
}
