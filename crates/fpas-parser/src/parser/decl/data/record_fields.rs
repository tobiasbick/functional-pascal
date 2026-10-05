//! Stored record fields and diagnostics for removed member declarations.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`

use crate::ast::*;
use crate::parser::Parser;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    pub(super) fn parse_record_type(&mut self, allow_member_visibility: bool) -> RecordType {
        let start = self.current_span();
        self.advance();
        let mut fields = Vec::new();
        while !self.check(&Token::End) && !self.at_end() {
            let position = self.pos;
            let visibility = self.parse_visibility(allow_member_visibility);
            if !self.recover_removed_record_member() {
                fields.push(self.parse_field_def(visibility));
            }
            if self.pos == position {
                self.advance();
            }
        }
        self.expect_named_end(&Token::Record);
        RecordType {
            fields,
            span: self.span_from(start),
        }
    }

    fn recover_removed_record_member(&mut self) -> bool {
        let routine = matches!(
            self.current_token(),
            Token::Function | Token::Procedure | Token::Pure
        );
        let static_routine = matches!(self.current_token(), Token::Ident(name) if name.eq_ignore_ascii_case("static"))
            && matches!(self.peek_token(), Token::Function | Token::Procedure);
        let accessor = matches!(self.current_token(), Token::Ident(name) if name.eq_ignore_ascii_case("property") || name.eq_ignore_ascii_case("event"))
            && matches!(self.peek_token(), Token::Ident(_));
        if !routine && !static_routine && !accessor {
            return false;
        }
        self.error_with_code(
            PARSE_EXPECTED_TOKEN,
            "Records contain stored fields, not methods, properties, or events",
            "Declare ordinary routines outside the record with an explicit record parameter. Store optional handlers in `Option of (HandlerType)` fields.",
            self.current_span(),
        );
        let routine = routine || static_routine;
        self.skip_removed_member_header();
        if routine
            && matches!(
                self.current_token(),
                Token::Begin | Token::Var | Token::Const
            )
        {
            self.skip_removed_member_body();
        }
        true
    }

    // Skips a member header through its `;`, ignoring semicolons inside parameter lists.
    fn skip_removed_member_header(&mut self) {
        let mut parentheses = 0usize;
        while !self.at_end() && !self.check(&Token::End) {
            match self.current_token() {
                Token::LParen => parentheses += 1,
                Token::RParen => parentheses = parentheses.saturating_sub(1),
                Token::Semicolon if parentheses == 0 => {
                    self.advance();
                    return;
                }
                _ => {}
            }
            self.advance();
        }
    }

    // Skips an inline member body through its `end function;` or `end procedure;`.
    fn skip_removed_member_body(&mut self) {
        while !self.at_end() {
            if self.check(&Token::End)
                && matches!(self.peek_token(), Token::Function | Token::Procedure)
            {
                self.advance();
                self.advance();
                self.eat(&Token::Semicolon);
                return;
            }
            self.advance();
        }
    }

    fn parse_field_def(&mut self, visibility: Visibility) -> FieldDef {
        let start = self.current_span();
        let (name, _) = match self.expect_ident() {
            Some(ident) => ident,
            None => {
                if !self.at_end() && !self.check(&Token::Semicolon) && !self.check(&Token::End) {
                    self.advance();
                }
                self.error_ident(start)
            }
        };
        self.expect(&Token::Colon);
        let type_expr = self.parse_type_expr();
        let default_value = if self.eat(&Token::ColonAssign) {
            Some(std::sync::Arc::new(self.parse_expression()))
        } else {
            None
        };
        self.expect_semi();
        FieldDef {
            name,
            type_expr,
            visibility,
            default_value,
            span: self.span_from(start),
        }
    }
}
