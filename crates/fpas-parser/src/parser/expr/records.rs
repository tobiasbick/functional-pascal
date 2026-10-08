//! Removed record literals and named record-update expression endings.
//!
//! **Documentation:** `docs/pascal/language/types/records.md`,
//! `docs/pascal/language/types/record-update.md`.

use super::super::Parser;
use crate::ast::{Expr, FieldInit, TypeExpr};
use fpas_diagnostics::codes::{PARSE_EMPTY_RECORD_UPDATE, PARSE_REMOVED_RECORD_LITERAL};
use fpas_lexer::{Span, Token};

impl Parser {
    /// Parses the initializer of a declaration whose type is written before it.
    ///
    /// A named declared type lets the removed-literal diagnostic name the constructor.
    pub(in crate::parser) fn parse_initializer(&mut self, type_expr: &TypeExpr) -> Expr {
        let outer = self.initializer_type.take();
        if let TypeExpr::Named { id, .. } = type_expr {
            self.initializer_type = Some((self.pos, id.parts.join(".")));
        }
        let value = self.parse_expression();
        self.initializer_type = outer;
        value
    }

    /// Rejects the removed `record Field := Value; end` literal and skips it.
    pub(super) fn reject_record_literal(&mut self) -> Expr {
        let type_name = self
            .initializer_type
            .as_ref()
            .filter(|(position, _)| *position == self.pos)
            .map(|(_, name)| name.clone());
        let start = self.advance().span;
        let fields = self.parse_field_init_list();
        if !self.at_enclosing_block_end() {
            self.eat(&Token::End);
        }
        let span = self.span_from(start);
        let arguments = if fields.is_empty() {
            "Field := Value".to_string()
        } else {
            fields
                .iter()
                .map(|field| format!("{} := ...", field.name))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let hint = match type_name {
            Some(name) => format!("Construct the record by its type name: `{name}({arguments})`."),
            None => format!(
                "Construct the record by its type name, for example `TypeName({arguments})`, where `TypeName` is the expected record type."
            ),
        };
        self.error_with_code(
            PARSE_REMOVED_RECORD_LITERAL,
            "Record literals `record ... end` are not supported",
            &hint,
            span,
        );
        Expr::Error(span)
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
