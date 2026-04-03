use crate::ast::*;
use crate::parser::Parser;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
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
        let (name, _) = self.expect_ident().unwrap_or(("_error_".into(), start));
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
                _ => fields.push(self.parse_field_def()),
            }
        }
        self.expect(&Token::End);
        RecordType {
            fields,
            methods,
            span: self.span_from(start),
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
                ("_error_".into(), start)
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
                ("_error_".into(), start)
            }
        };

        let fields = if self.eat(&Token::LParen) {
            let mut field_defs = Vec::new();
            while !self.check(&Token::RParen) && !self.at_end() {
                let field_start = self.current_span();
                let (field_name, _) = self
                    .expect_ident()
                    .unwrap_or(("_error_".into(), field_start));
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
