//! Binding initializers written as parenthesized lists: Pascal array constants `(A, B)` and Ada
//! aggregates `(X => 1, Y => 2)` or `(1, 2, 3)`.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/types/arrays.md`

use super::super::Parser;
use crate::ast::{Expr, TypeExpr};
use fpas_lexer::Token;

impl Parser {
    /// Parses an initializer. A parenthesized list, either a Pascal `(A, B)` array constant or an
    /// Ada aggregate `(X => 1, Y => 2)` (whose `=>` the lexer turned into `:=`), becomes an array
    /// literal or record placeholder. The flag tells whether an Ada aggregate was read.
    pub(super) fn parse_binding_value(
        &mut self,
        type_expr: Option<&TypeExpr>,
        type_text: Option<&str>,
        allow_paren_array: bool,
    ) -> (Expr, String, bool) {
        let start = self.current_span();
        let pascal_array = allow_paren_array && matches!(type_expr, Some(TypeExpr::Array(..)));
        if self.check(&Token::LParen) && (pascal_array || self.at_paren_aggregate()) {
            self.advance();
            let mut elements = Vec::new();
            let mut texts = Vec::new();
            let mut named = false;
            loop {
                let field = match (self.current_token(), self.peek_token()) {
                    (Token::Ident(field), Token::ColonAssign) => {
                        let field = field.clone();
                        self.advance();
                        self.advance();
                        named = true;
                        Some(field)
                    }
                    _ => None,
                };
                let element = self.parse_expression();
                let text = self.source_text(element.span()).to_owned();
                texts.push(match field {
                    Some(field) => format!("{field} := {text}"),
                    None => text,
                });
                elements.push(element);
                if !self.eat(&Token::Comma) {
                    break;
                }
            }
            self.expect(&Token::RParen);
            let span = self.span_from(start);
            let (value, text) = if named {
                let type_name = type_text.unwrap_or("TypeName");
                (
                    Expr::Error(span),
                    format!("{type_name}({})", texts.join(", ")),
                )
            } else {
                (
                    Expr::ArrayLiteral(elements, span),
                    format!("[{}]", texts.join(", ")),
                )
            };
            return (value, text, !pascal_array);
        }
        let value = match type_expr {
            Some(type_expr) => self.parse_initializer(type_expr),
            None => self.parse_expression(),
        };
        let text = self.source_text(self.span_from(start)).to_owned();
        (value, text, false)
    }

    /// True at `(Name := …` or at `(` whose top level holds a `,`; no parenthesized FPAS expression
    /// has either. A `:=` nested in a record update `(P with X := 1; end with)` does not count.
    fn at_paren_aggregate(&self) -> bool {
        let named = matches!(
            (
                self.tokens.get(self.pos + 1).map(|token| &token.token),
                self.tokens.get(self.pos + 2).map(|token| &token.token),
            ),
            (Some(Token::Ident(_)), Some(Token::ColonAssign))
        );
        if named {
            return true;
        }
        let mut depth = 0_usize;
        for token in self.tokens.iter().skip(self.pos) {
            match token.token {
                Token::LParen | Token::LBracket => depth += 1,
                Token::RParen | Token::RBracket => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return false;
                    }
                }
                Token::Comma if depth == 1 => return true,
                Token::Semicolon | Token::Eof => return false,
                _ => {}
            }
        }
        false
    }
}
