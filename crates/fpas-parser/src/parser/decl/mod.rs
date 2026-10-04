mod data;
mod names;
mod routines;
mod type_expr;
mod type_params;

use super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::{PARSE_INVALID_STATIC_PLACEMENT, PARSE_INVALID_VISIBILITY};
use fpas_lexer::Token;

impl Parser {
    pub(crate) fn parse_declarations(&mut self, allow_visibility: bool) -> Vec<Decl> {
        let mut decls = Vec::new();
        loop {
            let visibility = self.parse_visibility(allow_visibility);
            match self.current_token() {
                Token::Const => decls.extend(self.parse_binding_declaration(false, visibility)),
                Token::Var => decls.extend(self.parse_binding_declaration(true, visibility)),
                Token::Ident(_) if self.is_mutable_var_start() => {
                    self.reject_mutable_binding_keyword();
                    decls.extend(self.parse_binding_declaration(true, visibility));
                }
                Token::Type => {
                    decls.extend(self.parse_type_block(visibility, allow_visibility));
                }
                Token::Pure | Token::Function => {
                    decls.push(Decl::Function(self.parse_function_decl(visibility)));
                }
                Token::Procedure => {
                    decls.push(Decl::Procedure(self.parse_procedure_decl(visibility)));
                }
                Token::Ident(name)
                    if name.eq_ignore_ascii_case("static")
                        && matches!(self.peek_token(), Token::Function | Token::Procedure) =>
                {
                    if let Some(decl) = self.recover_invalid_static_decl() {
                        decls.push(decl);
                    }
                }
                Token::Ident(_) => {
                    let span = self.current_span();
                    self.error_with_code(fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN, "Each declaration requires its own keyword", "Repeat `type`, `const`, or `var` for every declaration, including `public` for every exported declaration.", span);
                    while !self.at_end()
                        && !self.check(&Token::Semicolon)
                        && !self.check(&Token::Begin)
                        && !self.check(&Token::End)
                    {
                        self.advance();
                    }
                    self.eat(&Token::Semicolon);
                }
                _ => break,
            }
        }
        decls
    }

    /// Diagnose an obsolete routine modifier and recover the ordinary declaration.
    fn recover_invalid_static_decl(&mut self) -> Option<Decl> {
        let span = self.current_span();
        self.error_with_code(
            PARSE_INVALID_STATIC_PLACEMENT,
            "Obsolete `static` routine modifier",
            "Remove `static` and declare the function or procedure at unit scope.",
            span,
        );
        self.advance(); // consume `static`
        match self.current_token() {
            Token::Function => Some(Decl::Function(
                self.parse_function_decl(Visibility::default()),
            )),
            Token::Procedure => Some(Decl::Procedure(
                self.parse_procedure_decl(Visibility::default()),
            )),
            _ => None,
        }
    }

    /// Parse an optional `public` declaration or record-member modifier.
    ///
    /// `docs/pascal/program-structure/units.md`: visibility modifiers are valid only in `unit`
    /// files, including modifiers nested inside record declarations.
    ///
    /// In a `program`, an invalid modifier still records the written visibility in the AST so the
    /// source intent is preserved; a diagnostic is always emitted.
    pub(super) fn parse_visibility(&mut self, allow_visibility: bool) -> Visibility {
        match self.current_token() {
            Token::Public => {
                let span = self.current_span();
                self.advance();
                if !allow_visibility {
                    self.error_with_code(
                        PARSE_INVALID_VISIBILITY,
                        "`public` is not valid in a `program` file",
                        "Remove `public`. Program-level declarations are not imported, so visibility modifiers are not allowed here.",
                        span,
                    );
                }
                Visibility::Public
            }
            _ => Visibility::Private,
        }
    }
}
