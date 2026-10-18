//! Delphi and C-family type applications in angle brackets, such as `TList<integer>`, and Delphi
//! generic constructors such as `TList<string>.Create`.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/types/generics.md`

use super::super::Parser;
use crate::QualifiedId;
use crate::ast::Expr;
use fpas_diagnostics::ForeignHabit;
use fpas_diagnostics::codes::{PARSE_EXPECTED_TOKEN, PARSE_FOREIGN_SYNTAX};
use fpas_lexer::Token;

impl Parser {
    /// Rejects a type application in angle brackets while recovering its arguments.
    pub(in crate::parser) fn reject_angle_type_arguments(&mut self, name: &QualifiedId) {
        self.advance();
        let mut arguments = Vec::new();
        if !self.check(&Token::Greater) {
            loop {
                let start = self.current_span();
                self.parse_type_expr();
                arguments.push(self.source_text(self.span_from(start)).to_owned());
                if !self.eat(&Token::Comma) {
                    break;
                }
            }
        }
        self.expect(&Token::Greater);
        let written = self.span_from(name.span);
        let found = self.source_text(written).to_owned();
        let type_name = name.parts.join(".");
        let argument = |index: usize, fallback: &str| {
            arguments
                .get(index)
                .cloned()
                .unwrap_or_else(|| fallback.to_owned())
        };
        let replacement = match type_name.to_ascii_lowercase().as_str() {
            "result" => format!("Result of ({}, {})", argument(0, "T"), argument(1, "E")),
            "dict" | "tdictionary" => {
                format!("dict of {} to {}", argument(0, "K"), argument(1, "V"))
            }
            "tlist" | "tobjectlist" | "array" => format!("array of {}", argument(0, "T")),
            _ if arguments.len() > 1 => format!("{type_name} of ({})", arguments.join(", ")),
            _ => format!("{type_name} of {}", argument(0, "T")),
        };
        self.report_foreign_habit(
            PARSE_EXPECTED_TOKEN,
            "Type applications use `of`, not angle brackets",
            ForeignHabit {
                found: &found,
                replacement: &replacement,
                explanation: "Generic types take their arguments after `of`; only routine declarations use `<T>` parameters.",
            },
            written,
        );
    }
}

impl Parser {
    /// True at a Delphi generic constructor such as `TList<string>.Create`.
    pub(in crate::parser) fn at_generic_constructor(&self) -> bool {
        if !matches!(self.current_token(), Token::Ident(_)) || self.peek_token() != &Token::Less {
            return false;
        }
        let mut index = self.pos + 2;
        while let Some(token) = self.tokens.get(index) {
            match token.token {
                Token::Ident(_) | Token::Comma | Token::Dot | Token::Of => index += 1,
                Token::Greater => {
                    return self
                        .tokens
                        .get(index + 1)
                        .is_some_and(|next| next.token == Token::Dot);
                }
                _ => return false,
            }
        }
        false
    }

    /// Skips `TList<T>.Create(…)` and reports it once with the FPAS value that replaces it.
    pub(in crate::parser) fn skip_generic_constructor(&mut self) -> Expr {
        let start = self.current_span();
        let type_name = match self.current_token() {
            Token::Ident(name) => name.clone(),
            _ => String::new(),
        };
        while !self.check(&Token::Greater) && !self.at_end() {
            self.advance();
        }
        self.advance();
        self.expect(&Token::Dot);
        self.expect_ident();
        if self.eat(&Token::LParen) {
            let mut depth = 1_usize;
            while depth > 0 && !self.at_end() {
                match self.current_token() {
                    Token::LParen => depth += 1,
                    Token::RParen => depth -= 1,
                    _ => {}
                }
                self.advance();
            }
        }
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        let replacement = match type_name.to_ascii_lowercase().as_str() {
            "tdictionary" | "tobjectdictionary" => "[:]",
            "tlist" | "tobjectlist" | "tqueue" | "tstack" | "tthreadedqueue" => "[]",
            _ => "TypeName of T(Field := …)",
        };
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "Delphi generic constructor is not FPAS syntax",
            ForeignHabit {
                found: &found,
                replacement,
                explanation: "FPAS collections are values without constructors: an empty array is `[]`, an empty dictionary `[:]`.",
            },
            written,
        );
        Expr::Error(written)
    }
}
