//! Scoped statement-list bodies for conditionals, case arms, and loops.
//!
//! **Documentation:** `docs/pascal/language/control-flow/README.md`.

use super::super::Parser;
use crate::ast::Stmt;
use crate::error::parse_error;
use fpas_diagnostics::codes::PARSE_EXPECTED_TOKEN;
use fpas_lexer::{Span, Token};

impl Parser {
    /// Parses a nonempty control body using its clause keyword as the scope anchor.
    pub(super) fn parse_control_body(&mut self, start: Span) -> Stmt {
        let statements = self.parse_statement_list();
        self.require_control_statements(&statements);
        Stmt::Block(statements, self.span_from(start))
    }

    /// Rejects an empty branch, case arm, or loop body with the canonical no-op spelling.
    pub(super) fn require_control_statements(&mut self, statements: &[Stmt]) {
        if statements.is_empty() && !self.nesting_limit_reached {
            let found = super::super::token_display(self.current_token());
            self.errors.push(
                parse_error(
                    PARSE_EXPECTED_TOKEN,
                    format!("Expected a statement in this body, found `{found}`"),
                    "Write `null;` when this branch, case arm, or loop intentionally does nothing.",
                    self.current_span(),
                )
                .with_expected_found("statement", found),
            );
        }
    }

    /// Requires a conditional ending and explains an unclosed nested `else if`.
    pub(super) fn expect_if_end(&mut self, nested_else_if: bool) {
        let error_count = self.errors.len();
        self.expect_block_end(&Token::If);
        if nested_else_if
            && self.errors.len() > error_count
            && let Some(error) = self.errors.get_mut(error_count)
        {
            error.help.get_or_insert_default().push_str(
                " `else if` starts a nested conditional and needs its own `end if;`. Use `elsif` to continue this conditional instead.",
            );
        }
    }
}
