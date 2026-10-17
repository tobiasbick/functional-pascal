//! Explicit conditional boundaries and `elsif` chains.
//!
//! **Documentation:** `docs/pascal/language/control-flow/if-then-else.md`.

use super::super::Parser;
use crate::ast::Stmt;
use crate::parser::foreign::PascalControl;
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
        let mut control = PascalControl::default();
        self.note_compound_body(&mut control);
        let errors_before = self.errors.len();
        let then_branch = Box::new(self.parse_control_body(then_start));
        self.note_else_terminator(&mut control, errors_before);
        let else_branch = if self.check(&Token::Elsif) {
            Some(Box::new(
                self.with_nesting(|parser| parser.parse_if_branch(false)),
            ))
        } else if self.check(&Token::Else) {
            let else_start = self.advance().span;
            self.note_compound_body(&mut control);
            Some(Box::new(self.parse_control_body(else_start)))
        } else {
            None
        };
        if owns_ending {
            let condition_text = self.source_text(condition.span());
            let form = if else_branch.is_some() {
                format!("if {condition_text} then …; else …; end if;")
            } else {
                format!("if {condition_text} then …; end if;")
            };
            self.finish_pascal_control(
                &Token::If,
                start,
                control,
                &form,
                contains_nested_else_if(else_branch.as_deref()),
            );
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
