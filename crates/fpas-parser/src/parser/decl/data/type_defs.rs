//! Individual type declarations, distinct type bodies, and enum bodies.
//!
//! **Documentation:** `docs/pascal/language/types/declaration-order.md` and
//! `docs/pascal/language/types/distinct-types.md`

use crate::ast::*;
use crate::parser::Parser;
use fpas_diagnostics::codes::{
    PARSE_EMPTY_ENUM_FIELD_LIST, PARSE_EXPECTED_IDENTIFIER, PARSE_EXPECTED_TOKEN,
    PARSE_TRAILING_ENUM_FIELD_SEPARATOR,
};
use fpas_lexer::Token;

impl Parser {
    /// Parses exactly one type after its own `type` keyword.
    pub(in super::super) fn parse_type_declaration(
        &mut self,
        visibility: Visibility,
        allow_member_visibility: bool,
    ) -> Option<Decl> {
        self.advance();
        if !self.can_start_declaration_definition() {
            self.error_with_code(
                PARSE_EXPECTED_IDENTIFIER,
                "Expected a type declaration after `type`",
                self.reserved_identifier_hint()
                    .unwrap_or("Add a declaration such as `type Count = integer;`."),
                self.current_span(),
            );
            return None;
        }
        Some(Decl::TypeDef(
            self.parse_type_def(visibility, allow_member_visibility),
        ))
    }

    /// Parses a type definition, including one recovered after a missing keyword.
    pub(in crate::parser::decl) fn parse_type_def(
        &mut self,
        visibility: Visibility,
        allow_member_visibility: bool,
    ) -> TypeDef {
        let start = self.current_span();
        let (name, _) = self.expect_ident_or_error(start);
        let type_params = self.parse_data_type_params();
        if self.check(&Token::Less) {
            let span = self.current_span();
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "Type parameters use `of`, not angle brackets",
                "Write `type Box of T = record ...` or `type Lookup of T = enum ...`.",
                span,
            );
            // consume to recover
            self.parse_type_params();
        }
        // Ada writes `type Name is …`; it is reported once together with its body below.
        let ada_is = !self.check(&Token::Equal) && self.eat(&Token::Is);
        if !ada_is {
            self.expect(&Token::Equal);
        }
        let body_start = self.current_span();
        let body = if self.at_delphi_class() {
            self.skip_delphi_class(&name)
        } else if self.at_enum_literal_list() {
            let (body, form) = self.parse_enum_literal_list(body_start);
            self.report_type_habit(start, &name, ada_is, &form);
            body
        } else if ada_is && self.at_ada_array_type() {
            let (body, form) = self.parse_ada_array_type(body_start);
            self.report_type_habit(start, &name, ada_is, &form);
            body
        } else {
            let body = self.parse_type_body(allow_member_visibility);
            if ada_is {
                let form = match body {
                    TypeBody::Record(_) => "record … end record",
                    TypeBody::Enum(_) => "enum … end enum",
                    _ => "…",
                };
                self.report_type_habit(start, &name, ada_is, form);
            }
            body
        };
        if !type_params.is_empty() && !matches!(body, TypeBody::Record(_) | TypeBody::Enum(_)) {
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "Type parameters are supported only on record and enum declarations",
                "Declare a generic record or enum, such as `type Lookup of T = enum Found(Value: T); Missing; end enum;`.",
                start,
            );
        }
        if !matches!(body, TypeBody::Record(_) | TypeBody::Enum(_))
            || !self.at_enclosing_block_end()
        {
            self.expect_semi();
        }
        TypeDef {
            name,
            type_params,
            body,
            visibility,
            span: self.span_from(start),
        }
    }

    fn parse_type_body(&mut self, allow_member_visibility: bool) -> TypeBody {
        if !matches!(
            self.current_token(),
            Token::Record | Token::Enum | Token::Distinct | Token::Type
        ) && self.is_subrange_body()
        {
            return self.parse_subrange_body();
        }
        match self.current_token() {
            Token::Record => TypeBody::Record(self.parse_record_type(allow_member_visibility)),
            Token::Enum | Token::Error => TypeBody::Enum(self.parse_enum_type()),
            Token::Distinct => {
                self.advance();
                TypeBody::Distinct(self.parse_type_expr())
            }
            Token::Type => {
                self.error_with_code(
                    PARSE_EXPECTED_TOKEN,
                    "`type X = type Y` is not a type declaration form",
                    "Write `type UserId = distinct integer;` for a distinct type, or `type UserId = integer;` for an alias.",
                    self.current_span(),
                );
                self.advance();
                TypeBody::Distinct(self.parse_type_expr())
            }
            _ => TypeBody::Alias(self.parse_type_expr()),
        }
    }

    fn parse_enum_type(&mut self) -> EnumType {
        let closer = self.current_token().clone();
        self.with_block_closer(closer, Self::parse_enum_type_inner)
    }

    fn parse_enum_type_inner(&mut self) -> EnumType {
        let start = self.current_span();
        let is_error = self.check(&Token::Error);
        let closer = if is_error { Token::Error } else { Token::Enum };
        self.advance();
        let mut members = Vec::new();
        if self.check(&Token::End) || self.at_end() {
            self.error_with_code(
                PARSE_EXPECTED_IDENTIFIER,
                "Expected at least one variant",
                "Add a variant between the declaration opener and its matching `end enum` or `end error`.",
                self.current_span(),
            );
        }
        while !self.check(&Token::End) && !self.at_end() {
            members.push(self.parse_enum_member(is_error));
        }
        self.expect_block_end(&closer);
        EnumType {
            is_error,
            members,
            span: self.span_from(start),
        }
    }

    fn parse_enum_member(&mut self, is_error: bool) -> EnumMember {
        let start = self.current_span();
        let (name, _) = match self.expect_ident() {
            Some(ident) => ident,
            None => {
                if !self.at_end() && !self.check(&Token::End) {
                    self.advance();
                }
                self.error_ident(start)
            }
        };

        let fields = if self.eat(&Token::LParen) {
            let mut field_defs = Vec::new();
            if self.check(&Token::RParen) {
                self.error_with_code(
                    PARSE_EMPTY_ENUM_FIELD_LIST,
                    "Enum variant field list must contain at least one field",
                    "Write `Variant;` for a fieldless variant or add a field such as `Variant(Value: integer);`.",
                    self.current_span(),
                );
            }
            while !self.check(&Token::RParen) && !self.at_end() {
                let field_start = self.current_span();
                let (field_name, _) = self
                    .expect_ident()
                    .unwrap_or_else(|| self.error_ident(field_start));
                self.expect(&Token::Colon);
                let type_expr = self.parse_type_expr();
                field_defs.push(EnumMemberField {
                    name: field_name,
                    type_expr,
                    span: self.span_from(field_start),
                });
                let separator_span = self.current_span();
                if !self.eat(&Token::Semicolon) {
                    break;
                }
                if self.check(&Token::RParen) {
                    self.error_with_code(
                        PARSE_TRAILING_ENUM_FIELD_SEPARATOR,
                        "Enum variant field list cannot end with `;`",
                        "Remove the trailing separator: write `Variant(First: integer; Second: integer);`.",
                        separator_span,
                    );
                    break;
                }
            }
            self.expect(&Token::RParen);
            field_defs
        } else {
            Vec::new()
        };

        let message = if is_error && self.eat(&Token::Equal) {
            if let Token::Str(text) = self.current_token() {
                let text = text.clone();
                if text.is_empty() {
                    self.error_with_code(
                        PARSE_EXPECTED_TOKEN,
                        "An error message must be nonempty",
                        "Omit `= ''` when the variant needs no literal message.",
                        self.current_span(),
                    );
                }
                self.advance();
                Some(text)
            } else {
                self.error_with_code(
                    PARSE_EXPECTED_TOKEN,
                    "An error variant message must be one nonempty string literal",
                    "Write `Failed = 'Operation failed';` or omit the message. Error variants have no integer backing values.",
                    self.current_span(),
                );
                if !self.at_end() && !self.check(&Token::Semicolon) && !self.check(&Token::End) {
                    self.advance();
                }
                None
            }
        } else {
            None
        };
        let value = if !is_error && fields.is_empty() && self.eat(&Token::Equal) {
            self.parse_enum_member_value()
        } else {
            None
        };

        self.expect_semi();
        EnumMember {
            message,
            name,
            value,
            fields,
            span: self.span_from(start),
        }
    }

    fn parse_enum_member_value(&mut self) -> Option<i64> {
        if let Token::Integer(value) = self.current_token() {
            let value = *value;
            let span = self.current_span();
            self.advance();
            let result = fpas_numeric::text::signed_integer(value, false);
            if result.is_none() {
                self.error_with_code(
                    fpas_diagnostics::codes::PARSE_INTEGER_LITERAL_OVERFLOW,
                    "Enum backing literal is outside the signed 64-bit range",
                    "Use an integer literal up to 9223372036854775807.",
                    span,
                );
            }
            result
        } else {
            let span = self.current_span();
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "Expected integer value for enum member",
                "Enum values must be integer literals.",
                span,
            );
            if !self.at_end() && !self.check(&Token::Semicolon) && !self.check(&Token::End) {
                self.advance();
            }
            None
        }
    }
}
