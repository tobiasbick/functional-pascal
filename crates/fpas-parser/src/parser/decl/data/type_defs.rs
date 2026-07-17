use crate::ast::*;
use crate::parser::Parser;
use fpas_diagnostics::codes::{PARSE_EXPECTED_TOKEN, PARSE_INVALID_STATIC_PLACEMENT};
use fpas_lexer::Token;

impl Parser {
    pub(in super::super) fn parse_type_block(&mut self, visibility: Visibility) -> Vec<Decl> {
        self.advance();
        let mut defs = Vec::new();
        while let Token::Ident(_) = self.current_token() {
            defs.push(Decl::TypeDef(self.parse_type_def(visibility)));
        }
        defs
    }

    fn parse_type_def(&mut self, visibility: Visibility) -> TypeDef {
        let start = self.current_span();
        let (name, _) = self
            .expect_ident()
            .unwrap_or_else(|| self.error_ident(start));
        if self.check(&Token::Less) {
            let span = self.current_span();
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "Generic type definitions are not supported. Only generic functions and procedures support type parameters.",
                "Remove `<...>` and use a generic function instead: `function Foo<T>(x: T): T`.",
                span,
            );
            // consume to recover
            self.parse_type_params();
        }
        self.expect(&Token::Equal);
        let body = self.parse_type_body();
        self.expect_semi();
        TypeDef {
            name,
            body,
            visibility,
            span: self.span_from(start),
        }
    }

    fn parse_type_body(&mut self) -> TypeBody {
        match self.current_token() {
            Token::Record => TypeBody::Record(self.parse_record_type()),
            Token::Enum => TypeBody::Enum(self.parse_enum_type()),
            _ => TypeBody::Alias(self.parse_type_expr()),
        }
    }

    fn parse_record_type(&mut self) -> RecordType {
        let start = self.current_span();
        self.advance();
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut properties = Vec::new();
        while !self.check(&Token::End) && !self.at_end() {
            match self.current_token() {
                Token::Function => {
                    methods.push(RecordMethod::Function(
                        self.parse_function_decl(Visibility::default()),
                    ));
                }
                Token::Procedure => {
                    methods.push(RecordMethod::Procedure(
                        self.parse_procedure_decl(Visibility::default()),
                    ));
                }
                Token::Static => {
                    if let Some(method) = self.parse_static_record_method() {
                        methods.push(method);
                    }
                }
                Token::Property => {
                    properties.push(self.parse_record_property());
                }
                _ => fields.push(self.parse_field_def()),
            }
        }
        self.expect(&Token::End);
        RecordType {
            fields,
            methods,
            properties,
            span: self.span_from(start),
        }
    }

    /// Parse `property Name: Type [read Getter] [write Setter];`.
    ///
    /// `read` / `write` are contextual identifiers, not reserved keywords.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-properties.md`
    fn parse_record_property(&mut self) -> RecordProperty {
        let start = self.current_span();
        self.advance(); // `property`
        let (name, _) = self
            .expect_ident()
            .unwrap_or_else(|| self.error_ident(start));
        self.expect(&Token::Colon);
        let type_expr = self.parse_type_expr();

        let mut read = None;
        let mut write = None;
        while matches!(self.current_token(), Token::Ident(_)) {
            let Some((accessor_kw, kw_span)) = self.expect_ident() else {
                break;
            };
            if accessor_kw.eq_ignore_ascii_case("read") {
                if read.is_some() {
                    self.error_with_code(
                        PARSE_EXPECTED_TOKEN,
                        "Duplicate `read` clause on property",
                        "Write `property Name: Type read Getter write Setter;`.",
                        kw_span,
                    );
                }
                let (getter, _) = self
                    .expect_ident()
                    .unwrap_or_else(|| self.error_ident(kw_span));
                read = Some(getter);
            } else if accessor_kw.eq_ignore_ascii_case("write") {
                if write.is_some() {
                    self.error_with_code(
                        PARSE_EXPECTED_TOKEN,
                        "Duplicate `write` clause on property",
                        "Write `property Name: Type read Getter write Setter;`.",
                        kw_span,
                    );
                }
                let (setter, _) = self
                    .expect_ident()
                    .unwrap_or_else(|| self.error_ident(kw_span));
                write = Some(setter);
            } else {
                self.error_with_code(
                    PARSE_EXPECTED_TOKEN,
                    "Expected `read` or `write` in a property declaration",
                    "Write `property Name: Type read Getter write Setter;`.",
                    kw_span,
                );
                break;
            }
        }

        self.expect_semi();
        RecordProperty {
            name,
            type_expr,
            read,
            write,
            span: self.span_from(start),
        }
    }

    /// Parse `static function …` inside a record. Rejects `static procedure` and bare `static`.
    fn parse_static_record_method(&mut self) -> Option<RecordMethod> {
        let static_span = self.current_span();
        self.advance(); // consume `static`
        match self.current_token() {
            Token::Function => Some(RecordMethod::StaticFunction(
                self.parse_function_decl(Visibility::default()),
            )),
            Token::Procedure => {
                self.error_with_code(
                    PARSE_INVALID_STATIC_PLACEMENT,
                    "`static procedure` is not supported",
                    "Only `static function` is allowed inside a record. Use an instance `procedure` with `Self`, or a `static function` that returns a value.",
                    static_span,
                );
                // Recover by parsing the procedure so the rest of the record stays sync'd.
                let _ = self.parse_procedure_decl(Visibility::default());
                None
            }
            _ => {
                self.error_with_code(
                    PARSE_INVALID_STATIC_PLACEMENT,
                    "`static` must be followed by `function` inside a record",
                    "Write `static function Name(...): ReturnType; begin … end;`.",
                    static_span,
                );
                None
            }
        }
    }

    fn parse_field_def(&mut self) -> FieldDef {
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
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect_semi();
        FieldDef {
            name,
            type_expr,
            default_value,
            span: self.span_from(start),
        }
    }

    fn parse_enum_type(&mut self) -> EnumType {
        let start = self.current_span();
        self.advance();
        let mut members = Vec::new();
        while !self.check(&Token::End) && !self.at_end() {
            members.push(self.parse_enum_member());
        }
        self.expect(&Token::End);
        EnumType {
            members,
            span: self.span_from(start),
        }
    }

    fn parse_enum_member(&mut self) -> EnumMember {
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
                if !self.eat(&Token::Semicolon) {
                    break;
                }
            }
            self.expect(&Token::RParen);
            field_defs
        } else {
            Vec::new()
        };

        let value = if fields.is_empty() && self.eat(&Token::Equal) {
            self.parse_enum_member_value()
        } else {
            None
        };

        self.expect_semi();
        EnumMember {
            name,
            value,
            fields,
            span: self.span_from(start),
        }
    }

    fn parse_enum_member_value(&mut self) -> Option<i64> {
        if let Token::Integer(value) = self.current_token() {
            let value = *value;
            self.advance();
            Some(value)
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
