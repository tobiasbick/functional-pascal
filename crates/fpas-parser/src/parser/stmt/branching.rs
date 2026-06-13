use super::super::Parser;
use crate::ast::*;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    pub(super) fn parse_if_stmt(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance();
        let condition = self.parse_expression();
        self.expect(&Token::Then);
        let then_branch = Box::new(self.parse_statement());
        let else_branch = if self.eat(&Token::Else) {
            Some(Box::new(self.parse_statement()))
        } else {
            None
        };
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            span: self.span_from(start),
        }
    }

    pub(super) fn parse_case_stmt(&mut self) -> Stmt {
        let start = self.current_span();
        self.advance();
        let expr = self.parse_expression();
        self.expect(&Token::Of);

        let mut arms = Vec::new();
        while !self.is_case_arm_end() {
            arms.push(self.parse_case_arm());
            if !self.eat(&Token::Semicolon) {
                if !self.is_case_arm_end() {
                    let span = self.current_span();
                    self.error_with_code(
                        PARSE_EXPECTED_TOKEN,
                        &format!(
                            "Expected `;` between case arms, found `{}`",
                            super::super::token_display(self.current_token()),
                        ),
                        "Insert `;` after the case arm body.",
                        span,
                    );
                }
                break;
            }
            if self.is_case_arm_end() {
                break;
            }
        }

        let else_body = if self.eat(&Token::Else) {
            let body = self.parse_statement_list();
            self.eat(&Token::Semicolon);
            Some(body)
        } else {
            None
        };

        self.expect(&Token::End);
        Stmt::Case {
            expr,
            arms,
            else_body,
            span: self.span_from(start),
        }
    }

    fn is_case_arm_end(&self) -> bool {
        matches!(self.current_token(), Token::End | Token::Else | Token::Eof)
    }

    fn parse_case_arm(&mut self) -> CaseArm {
        let start = self.current_span();
        let labels = self.parse_case_label_list();
        let guard = if self.eat(&Token::If) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect(&Token::Colon);
        let body = self.parse_statement();
        CaseArm {
            labels,
            guard,
            body,
            span: self.span_from(start),
        }
    }

    fn parse_case_label_list(&mut self) -> Vec<CaseLabel> {
        let mut labels = Vec::new();
        labels.push(self.parse_case_label());
        while self.eat(&Token::Comma) {
            labels.push(self.parse_case_label());
        }
        labels
    }

    fn parse_case_label(&mut self) -> CaseLabel {
        let start = self.current_span();

        match self.current_token() {
            Token::Ok | Token::Error | Token::Some | Token::None => {
                let variant = match self.current_token() {
                    Token::Ok => DestructureVariant::Ok,
                    Token::Error => DestructureVariant::Error,
                    Token::Some => DestructureVariant::Some,
                    Token::None => DestructureVariant::None,
                    _ => unreachable!(),
                };
                self.advance();
                let binding = if variant == DestructureVariant::None {
                    None
                } else {
                    self.expect(&Token::LParen);
                    let binding = self.expect_ident().map(|(name, _)| name);
                    self.expect(&Token::RParen);
                    binding
                };
                return CaseLabel::Destructure {
                    variant,
                    binding,
                    span: self.span_from(start),
                };
            }
            _ => {}
        }

        let start_expr = self.parse_expression();
        let end_expr = if self.eat(&Token::DotDot) {
            Some(self.parse_expression())
        } else {
            None
        };
        CaseLabel::Value {
            start: start_expr,
            end: end_expr,
            span: self.span_from(start),
        }
    }
}
