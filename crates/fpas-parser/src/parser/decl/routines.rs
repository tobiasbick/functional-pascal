use super::super::Parser;
use crate::ast::*;
use fpas_lexer::Token;

impl Parser {
    pub(super) fn parse_function_decl(&mut self, visibility: Visibility) -> Decl {
        let start = self.current_span();
        self.advance();
        let (name, _) = self.expect_ident().unwrap_or(("_error_".into(), start));
        let type_params = self.parse_type_params();
        self.expect(&Token::LParen);
        let params = self.parse_formal_param_list();
        self.expect(&Token::RParen);
        self.expect(&Token::Colon);
        let return_type = self.parse_type_expr();
        self.expect_semi();

        let body = self.parse_func_body();

        Decl::Function(FunctionDecl {
            name,
            type_params,
            params,
            return_type,
            body,
            visibility,
            span: self.span_from(start),
        })
    }

    pub(super) fn parse_procedure_decl(&mut self, visibility: Visibility) -> Decl {
        let start = self.current_span();
        self.advance();
        let (name, _) = self.expect_ident().unwrap_or(("_error_".into(), start));
        let type_params = self.parse_type_params();
        self.expect(&Token::LParen);
        let params = self.parse_formal_param_list();
        self.expect(&Token::RParen);
        self.expect_semi();

        let body = self.parse_func_body();

        Decl::Procedure(ProcedureDecl {
            name,
            type_params,
            params,
            body,
            visibility,
            span: self.span_from(start),
        })
    }

    fn parse_func_body(&mut self) -> FuncBody {
        if self.eat(&Token::Forward) {
            self.expect_semi();
            return FuncBody::Forward;
        }

        let nested = self.parse_nested_decls();
        self.expect(&Token::Begin);
        let stmts = self.parse_statement_list();
        self.expect(&Token::End);
        self.expect_semi();

        FuncBody::Block { nested, stmts }
    }

    pub(in crate::parser) fn parse_nested_decls(&mut self) -> Vec<Decl> {
        let mut decls = Vec::new();
        loop {
            match self.current_token() {
                Token::Function => decls.push(self.parse_function_decl(Visibility::default())),
                Token::Procedure => decls.push(self.parse_procedure_decl(Visibility::default())),
                _ => break,
            }
        }
        decls
    }

    pub(in crate::parser) fn parse_formal_param_list(&mut self) -> Vec<FormalParam> {
        let mut params = Vec::new();
        if self.check(&Token::RParen) {
            return params;
        }
        params.push(self.parse_formal_param());
        while self.eat(&Token::Semicolon) {
            params.push(self.parse_formal_param());
        }
        params
    }

    fn parse_formal_param(&mut self) -> FormalParam {
        let start = self.current_span();
        let mutable = self.eat(&Token::Mutable);
        let (name, _) = self.expect_ident().unwrap_or(("_error_".into(), start));
        self.expect(&Token::Colon);
        let type_expr = self.parse_type_expr();
        FormalParam {
            mutable,
            name,
            type_expr,
            span: self.span_from(start),
        }
    }
}
