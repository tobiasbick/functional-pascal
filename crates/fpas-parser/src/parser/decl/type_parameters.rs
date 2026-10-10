//! Generic parameter declarations for routines, records, and enums.
//! See `docs/pascal/language/types/generics.md`.

use crate::{TypeParam, parser::Parser};
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    /// Parses the optional angle-bracket parameters of a routine heading.
    pub(crate) fn parse_type_params(&mut self) -> Vec<TypeParam> {
        if !self.eat(&Token::Less) {
            return Vec::new();
        }
        let params = self.parse_type_parameter_list();
        self.expect(&Token::Greater);
        params
    }

    /// Parses data type parameters after `of`, with parentheses for multiple parameters.
    pub(super) fn parse_data_type_params(&mut self) -> Vec<TypeParam> {
        if !self.eat(&Token::Of) {
            return Vec::new();
        }
        let parenthesized = self.eat(&Token::LParen);
        let params = self.parse_type_parameter_list();
        if parenthesized {
            self.expect(&Token::RParen);
        }
        if parenthesized != (params.len() > 1) {
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "Type parameters require parentheses only when there is more than one parameter",
                "Write `type Box of T = record ...` or `type Pair of (K, V) = record ...`.",
                self.current_span(),
            );
        }
        params
    }

    fn parse_type_parameter_list(&mut self) -> Vec<TypeParam> {
        let mut params = vec![self.parse_single_type_param()];
        while self.eat(&Token::Comma) {
            params.push(self.parse_single_type_param());
        }
        params
    }

    fn parse_single_type_param(&mut self) -> TypeParam {
        let (name, _) = self
            .expect_ident()
            .unwrap_or_else(|| self.error_ident(self.current_span()));
        let constraint = if self.eat(&Token::Colon) {
            let constraint_name = match self.current_token() {
                Token::Comparable => "Comparable",
                Token::Numeric => "Numeric",
                Token::Printable => "Printable",
                _ => {
                    self.error_with_code(
                        PARSE_EXPECTED_TOKEN,
                        "Expected generic constraint `Comparable`, `Numeric`, or `Printable`",
                        "Use one of the supported generic constraints.",
                        self.current_span(),
                    );
                    return TypeParam {
                        name,
                        constraint: None,
                    };
                }
            }
            .to_owned();
            self.advance();
            Some(constraint_name)
        } else {
            None
        };
        TypeParam { name, constraint }
    }
}
