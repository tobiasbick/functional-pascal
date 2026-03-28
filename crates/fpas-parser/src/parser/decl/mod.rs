mod data;
mod routines;
mod type_expr;

use super::Parser;
use crate::ast::*;
use fpas_lexer::Token;

impl Parser {
    pub(crate) fn parse_declarations(&mut self) -> Vec<Decl> {
        let mut decls = Vec::new();
        loop {
            let visibility = self.parse_visibility();
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

    fn parse_visibility(&mut self) -> Visibility {
        match self.current_token() {
            Token::Public => {
                self.advance();
                Visibility::Public
            }
            Token::Private => {
                self.advance();
                Visibility::Private
            }
            _ => Visibility::default(),
        }
    }
}
