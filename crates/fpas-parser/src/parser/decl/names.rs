//! Recovery for declarations that incorrectly group multiple binding names.

use crate::parser::Parser;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    /// Diagnose grouped names and retain the following type for parser recovery.
    pub(in crate::parser) fn reject_grouped_names(
        &mut self,
        first: &str,
        example: fn(&str) -> String,
        separator: &str,
    ) {
        if !self.check(&Token::Comma) {
            return;
        }
        let span = self.current_span();
        let mut names = vec![first.to_owned()];
        while self.eat(&Token::Comma) {
            match self.current_token() {
                Token::Ident(name) => {
                    names.push(name.clone());
                    self.advance();
                }
                _ => break,
            }
        }
        let example = names
            .iter()
            .map(|name| example(name))
            .collect::<Vec<_>>()
            .join(separator);
        self.error_with_code(
            PARSE_EXPECTED_TOKEN,
            "Each declaration names exactly one binding",
            &format!("Write one declaration per name, for example `{example}`."),
            span,
        );
    }
}
