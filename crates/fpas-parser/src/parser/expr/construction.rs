//! Named fields in structural record construction.

use super::super::Parser;
use crate::ast::{Designator, DesignatorPart, Expr, FieldInit, QualifiedId};
use fpas_lexer::{Span, Token};

impl Parser {
    /// Recognize field assignment syntax at the start of a constructor's argument list.
    pub(in crate::parser) fn at_named_field(&self) -> bool {
        matches!(self.current_token(), Token::Ident(_)) && self.peek_token() == &Token::ColonAssign
    }

    /// Parse a name-only constructor target and comma-separated supplied fields.
    pub(in crate::parser) fn parse_record_construction(
        &mut self,
        target: Designator,
        start: Span,
    ) -> Expr {
        let type_name = QualifiedId {
            parts: target
                .parts
                .into_iter()
                .filter_map(|part| match part {
                    DesignatorPart::Ident(name, _) => Some(name),
                    DesignatorPart::Index(..) => None,
                })
                .collect(),
            span: target.span,
        };
        let mut fields = Vec::new();
        loop {
            let field_start = self.current_span();
            let (name, _) = self
                .expect_ident()
                .unwrap_or_else(|| self.error_ident(field_start));
            self.expect(&Token::ColonAssign);
            let value = self.parse_expression();
            fields.push(FieldInit {
                name,
                value,
                span: self.span_from(field_start),
            });
            if !self.eat(&Token::Comma) {
                break;
            }
        }
        self.expect(&Token::RParen);
        Expr::RecordConstruction {
            type_name,
            fields,
            span: self.span_from(start),
        }
    }
}
