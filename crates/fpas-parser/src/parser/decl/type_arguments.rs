//! Type-application delimiters and diagnostics for rejected angle-bracket forms.
//! See `docs/pascal/language/types/generics.md`.

use super::super::Parser;
use crate::{QualifiedId, TypeExpr};
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    /// Parses one unparenthesized argument or a parenthesized argument list after `of`.
    pub(in crate::parser) fn parse_named_type_arguments(&mut self) -> Vec<TypeExpr> {
        let parenthesized = self.eat(&Token::LParen);
        let mut arguments = vec![self.parse_type_expr()];
        if parenthesized {
            while self.eat(&Token::Comma) {
                arguments.push(self.parse_type_expr());
            }
            self.expect(&Token::RParen);
            if arguments.len() == 1 {
                self.error_with_code(
                    PARSE_EXPECTED_TOKEN,
                    "A single record type argument does not use parentheses",
                    "Write `Box of integer`; use parentheses for multiple arguments, as in `Pair of (integer, string)`.",
                    self.current_span(),
                );
            }
        }
        arguments
    }
    /// Parses the two required parenthesized arguments of a `Result` type.
    pub(super) fn parse_result_type_arguments(&mut self) -> (TypeExpr, TypeExpr) {
        let parenthesized = self.eat(&Token::LParen);
        if !parenthesized {
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "Result type arguments require parentheses after `of`",
                "Write `Result of (T, E)`, for example `Result of (integer, string)`.",
                self.current_span(),
            );
        }
        let ok_type = self.parse_type_expr();
        self.expect(&Token::Comma);
        let err_type = self.parse_type_expr();
        if parenthesized {
            self.expect(&Token::RParen);
        }
        (ok_type, err_type)
    }

    /// Rejects type applications in angle brackets while recovering their arguments.
    pub(super) fn reject_angle_type_arguments(&mut self, name: &QualifiedId) {
        let span = self.current_span();
        self.advance();
        let mut count = 0;
        if !self.check(&Token::Greater) {
            self.parse_type_expr();
            count += 1;
            while self.eat(&Token::Comma) {
                self.parse_type_expr();
                count += 1;
            }
        }
        self.expect(&Token::Greater);
        let name = name.parts.join(".");
        let example = if name.eq_ignore_ascii_case("result") {
            "Result of (T, E)".to_owned()
        } else if name.eq_ignore_ascii_case("dict") {
            "dict of K to V".to_owned()
        } else if count > 1 {
            let arguments = (1..=count)
                .map(|index| format!("T{index}"))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name} of ({arguments})")
        } else {
            format!("{name} of T")
        };
        self.error_with_code(
            PARSE_EXPECTED_TOKEN,
            "Type applications use `of`, not angle brackets",
            &format!("Write `{example}`. Routine declarations still use `<T>` parameters."),
            span,
        );
    }
}
