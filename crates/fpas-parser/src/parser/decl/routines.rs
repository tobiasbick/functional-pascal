//! Routine declarations (`function` / `procedure`).
//!
//! **Documentation:** `docs/pascal/language/functions/declarations.md` (from the repository root).

use super::super::Parser;
use crate::ast::*;
use fpas_lexer::{Span, Token};

impl Parser {
    /// Parse a function header: `function Name<T>(Params): RetType;`
    ///
    /// Consumes everything through the trailing semicolon and returns the
    /// parsed components. Shared by top-level declarations and record methods.
    fn parse_function_header(
        &mut self,
        allow_self_receiver: bool,
    ) -> (String, Vec<TypeParam>, Vec<FormalParam>, TypeExpr, Span) {
        let start = self.current_span();
        self.advance();
        let (name, _) = self
            .expect_ident()
            .unwrap_or_else(|| self.error_ident(start));
        let type_params = self.parse_type_params();
        self.expect(&Token::LParen);
        let params = self.parse_formal_param_list(allow_self_receiver);
        self.expect(&Token::RParen);
        self.expect(&Token::Colon);
        let return_type = self.parse_type_expr();
        self.expect_semi();
        (name, type_params, params, return_type, start)
    }

    /// Parse a procedure header: `procedure Name<T>(Params);`
    ///
    /// Consumes everything through the trailing semicolon.
    fn parse_procedure_header(
        &mut self,
        allow_self_receiver: bool,
    ) -> (String, Vec<TypeParam>, Vec<FormalParam>, Span) {
        let start = self.current_span();
        self.advance();
        let (name, _) = self
            .expect_ident()
            .unwrap_or_else(|| self.error_ident(start));
        let type_params = self.parse_type_params();
        self.expect(&Token::LParen);
        let params = self.parse_formal_param_list(allow_self_receiver);
        self.expect(&Token::RParen);
        self.expect_semi();
        (name, type_params, params, start)
    }

    pub(super) fn parse_function_decl(&mut self, visibility: Visibility) -> FunctionDecl {
        self.parse_function_decl_with_receiver(visibility, false)
    }

    pub(super) fn parse_record_function_decl(&mut self, visibility: Visibility) -> FunctionDecl {
        self.parse_function_decl_with_receiver(visibility, true)
    }

    fn parse_function_decl_with_receiver(
        &mut self,
        visibility: Visibility,
        allow_self_receiver: bool,
    ) -> FunctionDecl {
        self.with_nesting(|parser| {
            parser.parse_function_decl_inner(visibility, allow_self_receiver)
        })
    }

    fn parse_function_decl_inner(
        &mut self,
        visibility: Visibility,
        allow_self_receiver: bool,
    ) -> FunctionDecl {
        let (name, type_params, params, return_type, start) =
            self.parse_function_header(allow_self_receiver);
        let body = self.parse_func_body();
        FunctionDecl {
            name,
            type_params,
            params,
            return_type,
            body,
            visibility,
            span: self.span_from(start),
        }
    }

    pub(super) fn parse_procedure_decl(&mut self, visibility: Visibility) -> ProcedureDecl {
        self.parse_procedure_decl_with_receiver(visibility, false)
    }

    pub(super) fn parse_record_procedure_decl(&mut self, visibility: Visibility) -> ProcedureDecl {
        self.parse_procedure_decl_with_receiver(visibility, true)
    }

    fn parse_procedure_decl_with_receiver(
        &mut self,
        visibility: Visibility,
        allow_self_receiver: bool,
    ) -> ProcedureDecl {
        self.with_nesting(|parser| {
            parser.parse_procedure_decl_inner(visibility, allow_self_receiver)
        })
    }

    fn parse_procedure_decl_inner(
        &mut self,
        visibility: Visibility,
        allow_self_receiver: bool,
    ) -> ProcedureDecl {
        let (name, type_params, params, start) = self.parse_procedure_header(allow_self_receiver);
        let body = self.parse_func_body();
        ProcedureDecl {
            name,
            type_params,
            params,
            body,
            visibility,
            span: self.span_from(start),
        }
    }

    fn parse_func_body(&mut self) -> FuncBody {
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
                Token::Function => {
                    decls.push(Decl::Function(
                        self.parse_function_decl(Visibility::default()),
                    ));
                }
                Token::Procedure => {
                    decls.push(Decl::Procedure(
                        self.parse_procedure_decl(Visibility::default()),
                    ));
                }
                _ => break,
            }
        }
        decls
    }

    pub(in crate::parser) fn parse_formal_param_list(
        &mut self,
        allow_self_receiver: bool,
    ) -> Vec<FormalParam> {
        let mut params = Vec::new();
        if self.check(&Token::RParen) {
            return params;
        }
        params.push(self.parse_formal_param(allow_self_receiver));
        while self.eat(&Token::Semicolon) {
            if self.check(&Token::RParen) {
                let span = self.current_span();
                self.error_with_code(
                    fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN,
                    "Unexpected `;` before `)` in a parameter list",
                    "Remove the trailing semicolon, or add another parameter before `)`.",
                    span,
                );
                break;
            }
            params.push(self.parse_formal_param(false));
        }
        params
    }

    fn parse_formal_param(&mut self, allow_self_receiver: bool) -> FormalParam {
        let start = self.current_span();
        let mutable = self.eat(&Token::Mutable);
        let (name, _) = if allow_self_receiver && self.check(&Token::SelfKw) {
            let span = self.advance().span;
            ("Self".to_owned(), span)
        } else {
            self.expect_ident()
                .unwrap_or_else(|| self.error_ident(start))
        };
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
