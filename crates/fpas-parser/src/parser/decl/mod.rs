mod data;
mod routines;
mod type_expr;

use super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    pub(crate) fn parse_declarations(&mut self, allow_visibility: bool) -> Vec<Decl> {
        let mut decls = Vec::new();
        loop {
            let visibility = self.parse_visibility(allow_visibility);
            match self.current_token() {
                Token::Const => decls.extend(self.parse_const_block(visibility)),
                Token::Var => decls.extend(self.parse_var_block(false, visibility)),
                Token::Mutable => {
                    if self.peek_token() == &Token::Var {
                        decls.extend(self.parse_var_block(true, visibility));
                    } else {
                        break;
                    }
                }
                Token::Type => decls.extend(self.parse_type_block(visibility)),
                Token::Function => decls.push(self.parse_function_decl(visibility)),
                Token::Procedure => decls.push(self.parse_procedure_decl(visibility)),
                _ => break,
            }
        }
        decls
    }

    /// `docs/pascal/09-units.md`: visibility modifiers are valid only in `unit` files.
    fn parse_visibility(&mut self, allow_visibility: bool) -> Visibility {
        match self.current_token() {
            Token::Public => {
                let span = self.current_span();
                self.advance();
                if allow_visibility {
                    Visibility::Public
                } else {
                    self.error_with_code(
                        PARSE_EXPECTED_TOKEN,
                        "`public` is not valid in a `program` file",
                        "Remove `public`. Program-level declarations are not imported, so visibility modifiers are not allowed here.",
                        span,
                    );
                    Visibility::Public
                }
            }
            Token::Private => {
                let span = self.current_span();
                self.advance();
                if allow_visibility {
                    Visibility::Private
                } else {
                    self.error_with_code(
                        PARSE_EXPECTED_TOKEN,
                        "`private` is not valid in a `program` file",
                        "Remove `private`. Program-level declarations are not imported, so visibility modifiers are not allowed here.",
                        span,
                    );
                    Visibility::Public
                }
            }
            _ => Visibility::default(),
        }
    }
}
