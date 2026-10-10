//! Record field and routine parsing.

use crate::ast::*;
use crate::parser::Parser;
use fpas_diagnostics::codes::PARSE_INVALID_STATIC_PLACEMENT;
use fpas_lexer::Token;

impl Parser {
    /// Parses a record declaration with its matching `end record` ending.
    pub(super) fn parse_record_type(&mut self, allow_member_visibility: bool) -> RecordType {
        self.with_block_closer(Token::Record, |parser| {
            parser.parse_record_type_inner(allow_member_visibility)
        })
    }

    fn parse_record_type_inner(&mut self, allow_member_visibility: bool) -> RecordType {
        let start = self.current_span();
        self.advance();
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        while !self.check(&Token::End) && !self.at_end() {
            let visibility = self.parse_visibility(allow_member_visibility);
            match self.current_token() {
                Token::Function => {
                    methods.push(RecordMethod::Function(
                        self.parse_record_function_decl(visibility),
                    ));
                }
                Token::Procedure => {
                    methods.push(RecordMethod::Procedure(
                        self.parse_record_procedure_decl(visibility),
                    ));
                }
                Token::Static => {
                    if let Some(method) = self.parse_static_record_method(visibility) {
                        methods.push(method);
                    }
                }
                _ if self.at_removed_property() => self.reject_removed_property(),
                _ => fields.push(self.parse_field_def(visibility)),
            }
        }
        self.expect_block_end(&Token::Record);
        RecordType {
            fields,
            methods,
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
            Some(self.parse_initializer(&type_expr))
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
