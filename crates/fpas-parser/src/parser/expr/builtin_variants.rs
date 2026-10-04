//! Qualified constructors for the builtin Option and Result variant types.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use super::super::Parser;
use crate::ast::Expr;
use fpas_diagnostics::codes::PARSE_EXPECTED_EXPRESSION;
use fpas_lexer::Token;

impl Parser {
    /// Reject bare builtin variants with a concrete qualified replacement.
    pub(super) fn reject_unqualified_builtin(&mut self) -> Expr {
        let start = self.current_span();
        let option = matches!(self.current_token(), Token::Some | Token::None);
        self.error_with_code(
            PARSE_EXPECTED_EXPRESSION,
            "Builtin variant requires its type qualifier",
            if option {
                "Write `Option.Some(Value)` or `Option.None`."
            } else {
                "Write `Result.Ok(Value)` or `Result.Error(Message)`."
            },
            start,
        );
        if matches!(self.current_token(), Token::None) {
            self.advance();
        } else {
            self.parse_paren_wrapped_after_keyword();
        }
        Expr::Error(self.span_from(start))
    }

    /// Parse a builtin variant with its required type qualifier and positional payload.
    pub(super) fn parse_builtin_variant(&mut self) -> Expr {
        let start = self.current_span();
        let option = matches!(self.advance().token, Token::OptionKw);
        self.expect(&Token::Dot);
        match (option, self.current_token()) {
            (true, Token::Some) | (false, Token::Ok) | (false, Token::Error) => {
                let variant = self.current_token().clone();
                let (payload, _) = self.parse_paren_wrapped_after_keyword();
                let span = self.span_from(start);
                match variant {
                    Token::Some => Expr::OptionSome(Box::new(payload), span),
                    Token::Ok => Expr::ResultOk(Box::new(payload), span),
                    _ => Expr::ResultError(Box::new(payload), span),
                }
            }
            (true, Token::None) => {
                self.advance();
                Expr::OptionNone(self.span_from(start))
            }
            _ => {
                self.error_with_code(
                    PARSE_EXPECTED_EXPRESSION,
                    "Expected a variant of the qualified builtin type",
                    if option {
                        "Write `Option.Some(Value)` or `Option.None`."
                    } else {
                        "Write `Result.Ok(Value)` or `Result.Error(Message)`."
                    },
                    self.current_span(),
                );
                if !self.at_end() {
                    self.advance();
                }
                Expr::Error(self.span_from(start))
            }
        }
    }
}
