//! Routine declarations (`function` / `procedure`).
//!
//! **Documentation:** `docs/pascal/language/functions/declarations.md` (from the repository root).

use super::super::Parser;
use super::super::foreign::AdaHeaderParts;
use crate::ast::*;
use fpas_lexer::{Span, Token};

/// Name, type parameters and parameter list shared by function and procedure headers.
struct RoutineHead {
    name: String,
    type_params: Vec<TypeParam>,
    params: Vec<FormalParam>,
    /// Parameter list including parentheses; `None` when it was omitted.
    params_span: Option<Span>,
    start: Span,
}

impl Parser {
    /// Parse `function Name<T>(Params): RetType;`.
    ///
    /// Consumes everything through the trailing semicolon. Shared by top-level declarations
    /// and record methods. Returns whether an Ada `is` ended the header.
    fn parse_function_header(
        &mut self,
        allow_self_receiver: bool,
    ) -> (RoutineHead, TypeExpr, bool) {
        let head = self.parse_routine_head(allow_self_receiver);
        let ada_return = self.eat(&Token::Return);
        if !ada_return {
            self.expect(&Token::Colon);
        }
        let type_start = self.current_span();
        let return_type = self.parse_type_expr();
        let return_type_span = self.span_from(type_start);
        let ada_body =
            self.finish_routine_header(&head, "function", Some(return_type_span), ada_return);
        (head, return_type, ada_body)
    }

    /// Parse `procedure Name<T>(Params);`.
    ///
    /// Consumes everything through the trailing semicolon. Returns whether an Ada `is` ended
    /// the header.
    fn parse_procedure_header(&mut self, allow_self_receiver: bool) -> (RoutineHead, bool) {
        let head = self.parse_routine_head(allow_self_receiver);
        let ada_body = self.finish_routine_header(&head, "procedure", None, false);
        (head, ada_body)
    }

    /// Parses the routine keyword, name, type parameters and parameter list; a missing
    /// parameter list is reported once and recovered as `()`.
    fn parse_routine_head(&mut self, allow_self_receiver: bool) -> RoutineHead {
        let start = self.current_span();
        self.advance();
        let (name, name_span) = self
            .expect_ident()
            .unwrap_or_else(|| self.error_ident(start));
        let type_params = self.parse_type_params();
        if self.at_missing_parameter_list() {
            self.report_missing_parameter_list(&name, name_span);
            return RoutineHead {
                name,
                type_params,
                params: Vec::new(),
                params_span: None,
                start,
            };
        }
        let params_start = self.current_span();
        self.expect(&Token::LParen);
        let params = self.parse_formal_param_list(allow_self_receiver);
        self.expect(&Token::RParen);
        RoutineHead {
            name,
            type_params,
            params,
            params_span: Some(self.span_from(params_start)),
            start,
        }
    }

    /// Consumes the header terminator, reporting an Ada `return T` or `is` header once.
    /// Returns whether `is` ended the header.
    fn finish_routine_header(
        &mut self,
        head: &RoutineHead,
        keyword: &'static str,
        return_type: Option<Span>,
        ada_return: bool,
    ) -> bool {
        let ada_body = self.eat(&Token::Is);
        if ada_body || ada_return {
            self.report_ada_routine_header(
                head.start,
                AdaHeaderParts {
                    keyword,
                    name: &head.name,
                    params: head.params_span,
                    return_type,
                    is_body: ada_body,
                },
            );
        }
        if !ada_body {
            self.expect_semi();
        }
        ada_body
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
        let (head, return_type, ada_body) = self.parse_function_header(allow_self_receiver);
        let (requires, ensures, body) =
            self.parse_routine_rest(&head.name, ada_body, Token::Function);
        FunctionDecl {
            name: head.name,
            type_params: head.type_params,
            params: head.params,
            return_type,
            requires,
            ensures,
            body,
            visibility,
            span: self.span_from(head.start),
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
        let (head, ada_body) = self.parse_procedure_header(allow_self_receiver);
        let (requires, ensures, body) =
            self.parse_routine_rest(&head.name, ada_body, Token::Procedure);
        ProcedureDecl {
            is_release: false,
            name: head.name,
            type_params: head.type_params,
            params: head.params,
            requires,
            ensures,
            body,
            visibility,
            span: self.span_from(head.start),
        }
    }

    /// Parses contracts and the body; after an Ada header the Ada body is skipped instead.
    fn parse_routine_rest(
        &mut self,
        name: &str,
        ada_body: bool,
        kind: Token,
    ) -> (Vec<RequiresClause>, Vec<EnsuresClause>, FuncBody) {
        if ada_body {
            return (Vec::new(), Vec::new(), self.skip_ada_routine_body(name));
        }
        let requires = self.parse_requires_clauses();
        let ensures = self.parse_ensures_clauses();
        (requires, ensures, self.parse_func_body(kind))
    }

    fn parse_func_body(&mut self, kind: Token) -> FuncBody {
        self.with_block_closer(kind.clone(), |parser| {
            let mut nested = parser.parse_nested_decls();
            let mut stmts = Vec::new();
            if matches!(parser.current_token(), Token::Var | Token::Const) {
                stmts = parser.parse_routine_declaration_section();
                nested.extend(parser.parse_nested_decls());
            }
            parser.expect(&Token::Begin);
            stmts.extend(parser.parse_statement_list());
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
