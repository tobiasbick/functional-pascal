//! Generic declaration parameter lists.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use super::super::Parser;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    /// Parse an optional generic parameter list on a type or routine heading.
    pub(crate) fn parse_type_params(&mut self) -> Vec<crate::TypeParam> {
        if !self.eat(&Token::Of) {
            if self.check(&Token::Less) {
                self.error_with_code(
                    PARSE_EXPECTED_TOKEN,
                    "Generic parameters use `of (...)`",
                    "Write `function Identity of (T)(Value: T): T;` or `type Box of (T) = record ... end record;`.",
                    self.current_span(),
                );
                // Consume the obsolete heading for recovery without registering parameters.
                while !self.at_end()
                    && !self.check(&Token::Greater)
                    && !self.check(&Token::Semicolon)
                {
                    self.advance();
                }
                self.eat(&Token::Greater);
            }
            return Vec::new();
        }
        self.expect(&Token::LParen);
        let mut params = vec![self.parse_single_type_param()];
        while self.eat(&Token::Comma) {
            params.push(self.parse_single_type_param());
        }
        self.expect(&Token::RParen);
        params
    }

    fn parse_single_type_param(&mut self) -> crate::TypeParam {
        let (name, _) = self
            .expect_ident()
            .unwrap_or_else(|| self.error_ident(self.current_span()));
        let constraint = if self.eat(&Token::Colon) {
            let constraint_name = match self.current_token() {
                Token::Comparable => "Comparable",
                Token::Equatable => "Equatable",
                Token::Numeric => "Numeric",
                Token::Printable => "Printable",
                _ => {
                    self.error_with_code(PARSE_EXPECTED_TOKEN,
                        "Expected generic constraint `Equatable`, `Comparable`, `Numeric`, or `Printable`",
                        "Use one of the supported generic constraints.", self.current_span());
                    return crate::TypeParam { name, constraint: None };
                }
            }.to_owned();
            self.advance();
            Some(constraint_name)
        } else {
            None
        };
        crate::TypeParam { name, constraint }
    }
}
