//! Explicit conditional boundaries and `elsif` chains.
//!
//! **Documentation:** `docs/pascal/language/control-flow/if-then-else.md`.

use super::super::Parser;
use crate::ast::Stmt;
use fpas_lexer::Token;

impl Parser {
    /// Parses a conditional with scoped branches and one matching `end if`.
    pub(super) fn parse_if_stmt(&mut self) -> Stmt {
        self.with_block_closer(Token::If, |parser| parser.parse_if_branch(true))
    }

    fn parse_if_branch(&mut self, owns_ending: bool) -> Stmt {
        let start = self.current_span();
        self.advance();
        let condition = self.parse_expression();
        let then_start = self.expect(&Token::Then).unwrap_or(self.current_span());
        let then_branch = Box::new(self.parse_control_body(then_start));
        let else_branch = if self.check(&Token::Elsif) {
            Some(Box::new(
                self.with_nesting(|parser| parser.parse_if_branch(false)),
            ))
        } else if self.check(&Token::Else) {
            let else_start = self.advance().span;
            Some(Box::new(self.parse_control_body(else_start)))
        } else {
            None
        };
        if owns_ending {
            self.expect_if_end(contains_nested_else_if(else_branch.as_deref()));
        }
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            span: self.span_from(start),
        }
    }
}

fn contains_nested_else_if(branch: Option<&Stmt>) -> bool {
    match branch {
        Some(Stmt::If { else_branch, .. }) => contains_nested_else_if(else_branch.as_deref()),
        Some(Stmt::Block(statements, _)) => matches!(statements.first(), Some(Stmt::If { .. })),
        _ => false,
    }
}
