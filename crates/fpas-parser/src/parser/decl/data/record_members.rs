//! Record field, routine, property, and event parsing.

use crate::ast::*;
use crate::parser::Parser;
use fpas_diagnostics::codes::{
    PARSE_EXPECTED_TOKEN, PARSE_INVALID_STATIC_PLACEMENT, PARSE_INVALID_VISIBILITY,
};
use fpas_lexer::Token;

impl Parser {
    pub(super) fn parse_record_type(&mut self, allow_member_visibility: bool) -> RecordType {
        let start = self.current_span();
        self.advance();
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut properties = Vec::new();
        let mut events = Vec::new();
        while !self.check(&Token::End) && !self.at_end() {
            let visibility_span = matches!(self.current_token(), Token::Public | Token::Private)
                .then(|| self.current_span());
            let visibility = self.parse_visibility(allow_member_visibility);
            match self.current_token() {
                Token::Function => {
                    methods.push(RecordMethod::Function(self.parse_function_decl(visibility)));
                }
                Token::Procedure => {
                    methods.push(RecordMethod::Procedure(
                        self.parse_procedure_decl(visibility),
                    ));
                }
                Token::Static => {
                    if let Some(method) = self.parse_static_record_method(visibility) {
                        methods.push(method);
                    }
                }
                Token::Property => {
                    self.reject_record_property_or_event_visibility(visibility_span);
                    properties.push(self.parse_record_property());
                }
                Token::Event => {
                    self.reject_record_property_or_event_visibility(visibility_span);
                    events.push(self.parse_record_event());
                }
                _ => fields.push(self.parse_field_def(visibility)),
            }
        }
        self.expect(&Token::End);
        RecordType {
            fields,
            methods,
            properties,
            events,
            span: self.span_from(start),
        }
    }

    fn reject_record_property_or_event_visibility(
        &mut self,
        visibility_span: Option<fpas_lexer::Span>,
    ) {
        let Some(span) = visibility_span else {
            return;
        };
        self.error_with_code(
            PARSE_INVALID_VISIBILITY,
            "Visibility modifiers are not supported on record properties or events",
            "Remove the modifier. Record member visibility applies to fields, functions, and procedures.",
            span,
        );
    }

    /// Parse `property Name: Type [read Getter] [write Setter];`.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-properties.md`
    fn parse_record_property(&mut self) -> RecordProperty {
        let start = self.current_span();
        self.advance();
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
                let (getter, _) = self
                    .expect_ident()
                    .unwrap_or_else(|| self.error_ident(kw_span));
                if read.is_some() {
                    self.error_with_code(
                        PARSE_EXPECTED_TOKEN,
                        "Duplicate `read` clause on property",
                        "Write `property Name: Type read Getter write Setter;`.",
                        kw_span,
                    );
                } else {
                    read = Some(getter);
                }
            } else if accessor_kw.eq_ignore_ascii_case("write") {
                let (setter, _) = self
                    .expect_ident()
                    .unwrap_or_else(|| self.error_ident(kw_span));
                if write.is_some() {
                    self.error_with_code(
                        PARSE_EXPECTED_TOKEN,
                        "Duplicate `write` clause on property",
                        "Write `property Name: Type read Getter write Setter;`.",
                        kw_span,
                    );
                } else {
                    write = Some(setter);
                }
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

        if read.is_none() && write.is_none() {
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "Property must declare at least one of `read` or `write`",
                "Write `property Name: Type read Getter;` or add a `write` clause.",
                self.current_span(),
            );
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

    /// Parse `event Name: HandlerType read Getter write Setter;`.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    fn parse_record_event(&mut self) -> RecordEvent {
        let start = self.current_span();
        self.advance();
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
                let (getter, _) = self
                    .expect_ident()
                    .unwrap_or_else(|| self.error_ident(kw_span));
                if read.is_some() {
                    self.error_with_code(
                        PARSE_EXPECTED_TOKEN,
                        "Duplicate `read` clause on event",
                        "Write `event Name: HandlerType read Getter write Setter;`.",
                        kw_span,
                    );
                } else {
                    read = Some(getter);
                }
            } else if accessor_kw.eq_ignore_ascii_case("write") {
                let (setter, _) = self
                    .expect_ident()
                    .unwrap_or_else(|| self.error_ident(kw_span));
                if write.is_some() {
                    self.error_with_code(
                        PARSE_EXPECTED_TOKEN,
                        "Duplicate `write` clause on event",
                        "Write `event Name: HandlerType read Getter write Setter;`.",
                        kw_span,
                    );
                } else {
                    write = Some(setter);
                }
            } else {
                self.error_with_code(
                    PARSE_EXPECTED_TOKEN,
                    "Expected `read` or `write` in an event declaration",
                    "Write `event Name: HandlerType read Getter write Setter;`.",
                    kw_span,
                );
                break;
            }
        }

        if read.is_none() || write.is_none() {
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "Event requires both `read` and `write` accessors",
                "Write `event Name: HandlerType read Getter write Setter;`.",
                self.current_span(),
            );
        }

        self.expect_semi();
        RecordEvent {
            name,
            type_expr,
            read: read.unwrap_or_default(),
            write: write.unwrap_or_default(),
            span: self.span_from(start),
        }
    }

    fn parse_static_record_method(&mut self, visibility: Visibility) -> Option<RecordMethod> {
        let static_span = self.current_span();
        self.advance();
        match self.current_token() {
            Token::Function => Some(RecordMethod::StaticFunction(
                self.parse_function_decl(visibility),
            )),
            Token::Procedure => Some(RecordMethod::StaticProcedure(
                self.parse_procedure_decl(visibility),
            )),
            _ => {
                self.error_with_code(
                    PARSE_INVALID_STATIC_PLACEMENT,
                    "`static` must be followed by `function` or `procedure` inside a record",
                    "Write `static function Name(...): ReturnType; begin … end;` or `static procedure Name(...); begin … end;`.",
                    static_span,
                );
                None
            }
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
            Some(self.parse_expression())
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
