//! Record construction and named record-update expression endings.
//!
//! **Documentation:** `docs/pascal/language/types/record-update.md`.

use super::super::Parser;
use crate::ast::{Expr, FieldInit};
use fpas_diagnostics::codes::PARSE_EMPTY_RECORD_UPDATE;
use fpas_lexer::{Span, Token};

impl Parser {
    /// Parses a record literal, retaining its existing unnamed `end`.
    pub(super) fn parse_record_literal(&mut self) -> Expr {
        let start = self.advance().span;
        let fields = self.parse_field_init_list();
        if self.at_enclosing_block_end() {
            self.error_with_code(
                fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN,
                "Expected `end` before the enclosing block ending",
                "Close this record literal with `end` before its enclosing construct.",
                self.current_span(),
            );
        } else {
            self.expect(&Token::End);
        }
        Expr::RecordLiteral {
            fields,
            span: self.span_from(start),
        }
    }

    /// Parses `base with Field := Value; … end with` without consuming a terminator.
    pub(super) fn parse_record_update(&mut self, base: Expr, start: Span) -> Expr {
        self.with_block_closer(Token::With, |parser| {
            parser.advance();
            if parser.check(&Token::End) {
                parser.error_with_code(
                    PARSE_EMPTY_RECORD_UPDATE,
                    "Record update requires at least one field assignment",
                    "Add a field assignment, for example `Value with X := 1; end with`, or use the original value directly.",
                    parser.current_span(),
                );
            }
            let fields = parser.parse_field_init_list();
            parser.expect_expression_end(&Token::With);
            Expr::RecordUpdate {
                base: Box::new(base),
                fields,
                span: parser.span_from(start),
            }
        })
    }

    fn parse_field_init_list(&mut self) -> Vec<FieldInit> {
        let mut fields = Vec::new();
        while !self.is_expression_recovery_boundary() && !self.is_stmt_list_end() {
            let field_position = self.pos;
            let field_start = self.current_span();
            let (name, _) = self
                .expect_ident()
                .unwrap_or_else(|| self.error_ident(field_start));
            self.expect(&Token::ColonAssign);
            let value = self.parse_expression();
            self.expect_semi();
            fields.push(FieldInit {
                name,
                value,
                span: self.span_from(field_start),
            });
            // Leave parent delimiters available after a missing field value.
            if self.pos == field_position {
                break;
            }
        }
        fields
    }
}
