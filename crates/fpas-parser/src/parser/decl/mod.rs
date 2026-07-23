mod data;
mod routines;
mod type_expr;

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
                Token::Const => decls.extend(self.parse_const_block(visibility)),
                Token::Var => decls.extend(self.parse_var_block(false, visibility)),
                Token::Mutable if self.is_mutable_var_start() => {
                    decls.extend(self.parse_var_block(true, visibility));
                }
                Token::Mutable => break,
                Token::Type => decls.extend(self.parse_type_block(visibility)),
                Token::Function => {
                    decls.push(Decl::Function(self.parse_function_decl(visibility)));
                }
                Token::Procedure => {
                    decls.push(Decl::Procedure(self.parse_procedure_decl(visibility)));
                }
                Token::Static => {
                    self.recover_invalid_static_decl();
                }
                _ => break,
            }
        }
        decls
    }

    /// `static` is only valid on a function or procedure inside a record type body.
    fn recover_invalid_static_decl(&mut self) {
        let span = self.current_span();
        self.error_with_code(
            PARSE_INVALID_STATIC_PLACEMENT,
            "`static` is only valid on a function or procedure declared inside a record",
            "Move the routine into a `record … end` body and write `static function Name(...): T;` or `static procedure Name(...);`.",
            span,
        );
        self.advance(); // consume `static`
        match self.current_token() {
            Token::Function => {
                let _ = self.parse_function_decl(Visibility::default());
            }
            Token::Procedure => {
                let _ = self.parse_procedure_decl(Visibility::default());
            }
            _ => {}
        }
    }

    /// `docs/pascal/program-structure/units.md`: visibility modifiers are valid only in `unit` files.
    ///
    /// In a `program`, an invalid modifier still records the written visibility in the AST so the
    /// source intent is preserved; a diagnostic is always emitted.
    fn parse_visibility(&mut self, allow_visibility: bool) -> Visibility {
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
            Token::Private => {
                let span = self.current_span();
                self.advance();
                if !allow_visibility {
                    self.error_with_code(
                        PARSE_INVALID_VISIBILITY,
                        "`private` is not valid in a `program` file",
                        "Remove `private`. Program-level declarations are not imported, so visibility modifiers are not allowed here.",
                        span,
                    );
                }
                Visibility::Private
            }
            _ => Visibility::default(),
        }
    }
}
