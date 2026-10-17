//! Explicit case-arm boundaries and scoped statement-list bodies.
//!
//! **Documentation:** `docs/pascal/language/control-flow/case-of-intro.md`.

mod labels;

use super::super::Parser;
use crate::ast::*;
use crate::parser::foreign::PascalControl;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::Token;

impl Parser {
    /// Parses `when` arms and an optional catch-all before the matching `end case`.
    pub(super) fn parse_case_stmt(&mut self) -> Stmt {
        self.with_block_closer(Token::Case, Self::parse_case_stmt_inner)
    }

    fn parse_case_stmt_inner(&mut self) -> Stmt {
        let start = self.advance().span;
        let expr = self.parse_expression();
        self.expect(&Token::Of);

        let mut arms = Vec::new();
        if matches!(self.current_token(), Token::Else | Token::End | Token::Eof) {
            self.error_with_code(
                PARSE_EXPECTED_TOKEN,
                "Expected at least one case arm",
                "Add an arm such as `when 1: null;` after `of`.",
                self.current_span(),
            );
        }
        let mut control = PascalControl::default();
        let mut labels = Vec::new();
        while self.check(&Token::When) || !self.is_stmt_list_end() {
            let label_start = self.current_span();
            let pascal_arm = !self.eat(&Token::When);
            if pascal_arm {
                self.note_arm_without_when(&mut control);
            }
            let arm = self.parse_case_arm(label_start, pascal_arm);
            labels.push(self.source_text(arm.labels_span).to_owned());
            arms.push(arm.arm);
        }

        let else_body = if self.eat(&Token::Else) {
            let statements = self.parse_statement_list();
            self.require_control_statements(&statements);
            Some(statements)
        } else {
            None
        };
        let mut form = format!("case {} of", self.source_text(expr.span()));
        for label in &labels {
            form.push_str(&format!(" when {label}: …;"));
        }
        if else_body.is_some() {
            form.push_str(" else …;");
        }
        form.push_str(" end case;");
        self.finish_pascal_control(&Token::Case, start, control, &form, false);
        Stmt::Case {
            expr,
            arms,
            else_body,
            span: self.span_from(start),
        }
    }

    /// Parses one arm after its `when`; a Pascal arm without `when` has exactly one statement.
    fn parse_case_arm(&mut self, start: fpas_lexer::Span, pascal_arm: bool) -> ParsedCaseArm {
        let labels_start = self.current_span();
        let labels = self.parse_case_label_list();
        let labels_span = self.span_from(labels_start);
        let guard = if self.eat(&Token::If) {
            Some(self.parse_expression())
        } else {
            None
        };
        let body_start = self.expect(&Token::Colon).unwrap_or(self.current_span());
        let body = if pascal_arm {
            let statement = self.parse_terminated_statement();
            Stmt::Block(vec![statement], self.span_from(body_start))
        } else {
            self.parse_control_body(body_start)
        };
        ParsedCaseArm {
            arm: CaseArm {
                labels,
                guard,
                body,
                span: self.span_before_terminator(start),
            },
            labels_span,
        }
    }
}

/// A parsed case arm and the source span of its labels.
struct ParsedCaseArm {
    arm: CaseArm,
    labels_span: fpas_lexer::Span,
}
