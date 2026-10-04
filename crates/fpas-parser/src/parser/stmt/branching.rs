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
        let then_branch = self.parse_statement_body();
        let mut elsif_branches = Vec::new();
        while self.eat(&Token::Elsif) {
            let condition = self.parse_expression();
            self.expect(&Token::Then);
            let body = self.parse_statement_body();
            elsif_branches.push((condition, body));
        }
        let else_branch = if self.eat(&Token::Else) {
            Some(self.parse_statement_body())
        } else {
            None
        };
        self.expect_named_end(&Token::If);
        Stmt::If {
            condition,
            then_branch,
            elsif_branches,
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
        if matches!(self.current_token(), Token::Else | Token::End | Token::Eof) {
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "Expected at least one case arm",
                "Add a case arm such as `when 1: Value := 1;` after `of`.",
                self.current_span(),
            );
        }
        while self.check(&Token::When) {
            arms.push(self.parse_case_arm());
        }
        if arms.is_empty() && !matches!(self.current_token(), Token::End | Token::Else | Token::Eof)
        {
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "Case arms must start with `when`",
                "Write `when Pattern: Statement;` after `of`.",
                self.current_span(),
            );
            while !self.at_end() && !self.check(&Token::End) {
                self.advance();
            }
        }

        let else_body = if self.eat(&Token::Else) {
            let body = self.parse_statement_list();
            Some(body)
        } else {
            None
        };

        self.expect_named_end(&Token::Case);
        Stmt::Case {
            expr,
            arms,
            else_body,
            span: self.span_from(start),
        }
    }

    fn parse_case_arm(&mut self) -> CaseArm {
        let start = self.current_span();
        self.expect(&Token::When);
        let labels = self.parse_case_label_list();
        let guard = if self.eat(&Token::If) {
            Some(self.parse_expression())
        } else {
            None
        };
        self.expect(&Token::Colon);
        let body = *self.parse_statement_body();
        CaseArm {
            labels,
            guard,
            body,
            span: self.span_from(start),
        }
    }

    /// Parse the same recursive label list for statement and expression cases.
    pub(in crate::parser) fn parse_case_label_list(&mut self) -> Vec<CaseLabel> {
        let mut labels = vec![self.parse_pattern()];
        while self.eat(&Token::Comma) {
            labels.push(self.parse_pattern());
        }
        labels
    }
}
