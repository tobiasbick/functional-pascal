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
        let body = self.parse_func_body(Token::Function);
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
        let body = self.parse_func_body(Token::Procedure);
        ProcedureDecl {
            name,
            type_params,
            params,
            body,
            visibility,
            span: self.span_from(start),
        }
    }

    fn parse_func_body(&mut self, kind: Token) -> FuncBody {
        self.with_block_closer(kind.clone(), |parser| {
            let nested = parser.parse_nested_decls();
            parser.expect(&Token::Begin);
            let stmts = parser.parse_statement_list();
            if parser.expect_block_end(&kind) {
                parser.expect_semi();
            }
            FuncBody::Block { nested, stmts }
        })
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
}
