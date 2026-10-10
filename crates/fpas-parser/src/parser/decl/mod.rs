mod data;
mod parameters;
mod routines;
mod type_arguments;
mod type_expr;
mod type_parameters;

use super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::{PARSE_INVALID_STATIC_PLACEMENT, PARSE_INVALID_VISIBILITY};
use fpas_lexer::Token;

impl Parser {
    /// Parses an ordered list with an explicit keyword on each declaration.
    pub(crate) fn parse_declarations(&mut self, allow_visibility: bool) -> Vec<Decl> {
        let mut decls = Vec::new();
        loop {
            let visibility = self.parse_visibility(allow_visibility);
            match self.current_token() {
                Token::Const => decls.extend(self.parse_const_declaration(visibility)),
                Token::Var => decls.extend(self.parse_variable_declaration(visibility)),
                Token::Ident(_) if self.is_mutable_var_start() => {
                    self.reject_mutable_binding();
                    decls.extend(self.parse_variable_declaration(visibility));
                }
                Token::Type => {
                    decls.extend(self.parse_type_declaration(visibility, allow_visibility));
                }
                Token::Function => {
                    decls.push(Decl::Function(self.parse_function_decl(visibility)));
                }
                Token::Procedure => {
                    decls.push(Decl::Procedure(self.parse_procedure_decl(visibility)));
                }
                Token::Static => {
                    if let Some(decl) = self.recover_invalid_static_decl() {
                        decls.push(decl);
                    }
                }
                _ => {
                    let Some(declaration) = self.recover_unprefixed_declaration(
                        decls.last(),
                        visibility,
                        allow_visibility,
                    ) else {
                        break;
                    };
                    decls.push(declaration);
                }
            }
        }
        decls
    }

    /// `static` is only valid on a function or procedure inside a record type body.
    fn recover_invalid_static_decl(&mut self) -> Option<Decl> {
        let span = self.current_span();
        self.error_with_code(
            PARSE_INVALID_STATIC_PLACEMENT,
            "`static` is only valid on a function or procedure declared inside a record",
            "Move the routine into a `record … end` body and write `static function Name(...): T;` or `static procedure Name(...);`.",
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
