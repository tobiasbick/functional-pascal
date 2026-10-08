//! Rejection of the removed `property` record member.
//!
//! **Documentation:** `docs/pascal/language/types/record-methods.md`

use crate::parser::Parser;
use fpas_diagnostics::codes::PARSE_REMOVED_PROPERTY;
use fpas_lexer::Token;

impl Parser {
    /// True when the current member starts with the former `property Name` form.
    pub(super) fn at_removed_property(&self) -> bool {
        matches!(self.current_token(), Token::Ident(word) if word.eq_ignore_ascii_case("property"))
            && matches!(self.peek_token(), Token::Ident(_))
    }

    /// Skips `property Name: Type [read Getter] [write Setter];` and reports its replacement.
    pub(super) fn reject_removed_property(&mut self) {
        let start = self.advance().span;
        let name = self.expect_ident().map(|(name, _)| name);
        if self.eat(&Token::Colon) {
            let _ = self.parse_type_expr();
        }
        let mut read = None;
        let mut write = None;
        while matches!(self.current_token(), Token::Read | Token::Write) {
            let is_read = self.check(&Token::Read);
            self.advance();
            let accessor = self.expect_ident().map(|(accessor, _)| accessor);
            if is_read {
                read = read.or(accessor);
            } else {
                write = write.or(accessor);
            }
        }
        self.expect_semi();
        let span = self.span_from(start);
        let mut calls = Vec::new();
        if let Some(getter) = &read {
            calls.push(format!("read with `Value.{getter}()`"));
        }
        if let Some(setter) = &write {
            calls.push(format!("write with `Value.{setter}(NewValue)`"));
        }
        let hint = if calls.is_empty() {
            "Remove the property and declare the instance methods callers need.".to_string()
        } else {
            format!(
                "Remove the property and call its accessor methods directly: {}. Make an accessor `public` if importers used the property.",
                calls.join("; ")
            )
        };
        let message = match name {
            Some(name) => {
                format!("Record properties are not supported; `{name}` is declared with `property`")
            }
            None => "Record properties are not supported".to_string(),
        };
        self.error_with_code(PARSE_REMOVED_PROPERTY, &message, &hint, span);
    }
}
