use super::super::Parser;
use crate::ast::*;
use crate::parser::token_display;
use fpas_diagnostics::codes::{PARSE_EXPECTED_EXPRESSION, PARSE_INVALID_STATEMENT_START};
use fpas_lexer::Token;

impl Parser {
    /// Parse `go <call-expression>`.
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    pub(crate) fn parse_go_stmt(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance(); // consume `go`
        let expr = self.parse_expression();
        Stmt::Go {
            expr,
            span: self.span_from(start),
        }
    }

    /// Parse `select ... end`.
    ///
    /// ```text
    /// select
    ///   case Msg: string from Ch1:
    ///     WriteLn(Msg);
    ///   case Num: integer from Ch2:
    ///     WriteLn(Num);
    ///   default:
    ///     WriteLn('none');
    /// end
    /// ```
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    pub(crate) fn parse_select_stmt(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance(); // consume `select`

        let mut arms = Vec::new();
        let mut default_body = None;

        loop {
            match self.current_token() {
                Token::End => {
                    self.advance();
                    break;
                }
                Token::Eof => {
                    self.error_with_code(
                        PARSE_INVALID_STATEMENT_START,
                        "Unexpected end of file inside `select`",
                        "Add `end` to close the `select` statement.",
                        self.current_span(),
                    );
                    break;
                }
                Token::Case => {
                    arms.push(self.parse_select_arm());
                    self.eat(&Token::Semicolon);
                }
                Token::Default => {
                    self.advance(); // consume `default`
                    self.expect(&Token::Colon);
                    let body = self.parse_statement_list();
                    default_body = Some(body);
                    self.eat(&Token::Semicolon);
                }
                _ => {
                    self.error_with_code(
                        PARSE_INVALID_STATEMENT_START,
                        &format!(
                            "Unexpected token `{}` in `select`",
                            token_display(self.current_token())
                        ),
                        "Expected `case`, `default`, or `end`.",
                        self.current_span(),
                    );
                    self.advance();
                }
            }
        }

        if arms.is_empty() && default_body.is_none() {
            self.error_with_code(
                PARSE_INVALID_STATEMENT_START,
                "Empty `select` is not allowed",
                "Add at least one `case` arm or a `default` arm to the `select` statement.",
                self.span_from(start),
            );
        }

        Stmt::Select {
            arms,
            default_body,
            span: self.span_from(start),
        }
    }

    /// Parse a single select arm: `case Binding: Type from ChannelExpr: Body`.
    fn parse_select_arm(&mut self) -> SelectArm {
        let start = self.current_span();
        self.advance(); // consume `case`

        let binding = match self.current_token() {
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => {
                self.error_with_code(
                    PARSE_EXPECTED_EXPRESSION,
                    "Expected binding identifier in select arm",
                    "Syntax: `case Name: Type from Channel: body`.",
                    self.current_span(),
                );
                String::from("_")
            }
        };

        self.expect(&Token::Colon);
        let type_expr = self.parse_type_expr();
        self.expect(&Token::From);
        let channel = self.parse_expression();
        self.expect(&Token::Colon);

        let body = self.parse_statement();

        SelectArm {
            binding,
            type_expr,
            channel,
            body,
            span: self.span_from(start),
        }
    }
}
