//! Integer subrange declaration bodies.
//!
//! **Documentation:** `docs/pascal/language/types/subranges.md`

use crate::{TypeBody, parser::Parser};
use fpas_lexer::Token;

impl Parser {
    /// Distinguishes a range body from an alias without consuming either bound.
    pub(super) fn is_subrange_body(&self) -> bool {
        let mut blocks = 0usize;
        let mut parentheses = 0usize;
        let mut after_end = false;
        for token in &self.tokens[self.pos..] {
            match token.token {
                Token::If | Token::Case if !after_end => blocks += 1,
                Token::End => blocks = blocks.saturating_sub(1),
                Token::LParen => parentheses += 1,
                Token::RParen => parentheses = parentheses.saturating_sub(1),
                Token::DotDot if blocks == 0 && parentheses == 0 => return true,
                Token::Semicolon if blocks == 0 && parentheses == 0 => return false,
                _ => {}
            }
            after_end = token.token == Token::End;
        }
        false
    }

    /// Parses expressions; semantic analysis requires static integer values.
    pub(super) fn parse_subrange_body(&mut self) -> TypeBody {
        let lower = Box::new(self.parse_expression());
        self.expect(&Token::DotDot);
        let upper = Box::new(self.parse_expression());
        TypeBody::Subrange { lower, upper }
    }
}
