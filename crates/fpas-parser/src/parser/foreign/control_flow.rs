//! Pascal and Ada control-flow habits: `begin … end` branch and loop bodies, missing `;` before
//! `else`, missing named endings, Pascal `for I := …`, Ada `for I in A .. B loop` and
//! `while C loop`, Pascal case arms without `when`, and Delphi `try … except/finally`.
//!
//! Signs of Pascal syntax are collected while a statement is parsed. When its named ending is
//! missing, the follow-up errors of that statement collapse into one diagnostic. A statement that
//! still closes with its FPAS ending keeps the ordinary diagnostics, because `begin … end` inside
//! a branch is valid FPAS.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings),
//! `docs/pascal/language/control-flow/README.md`

use super::super::{Parser, token_display};
use crate::ast::Stmt;
use fpas_diagnostics::ForeignHabit;
use fpas_diagnostics::codes::{PARSE_EXPECTED_TOKEN, PARSE_FOREIGN_SYNTAX};
use fpas_lexer::{Span, Token};

/// Tokens a scan for a Delphi `except`/`finally` looks ahead at most.
const MAX_TRY_SCAN: usize = 4096;

/// Signs of Pascal syntax seen while one control statement was parsed.
#[derive(Debug, Default)]
pub(in crate::parser) struct PascalControl {
    /// A branch or loop body started with `begin`.
    compound_body: bool,
    /// Index of the error for a missing `;` directly before `else`.
    pub(super) else_terminator_error: Option<usize>,
    /// The `for` header omitted the loop variable's type.
    untyped_for: bool,
    /// Pascal case arms were written without `when`.
    arms_without_when: bool,
    /// Branch and loop bodies in source order, for the rewrite fix.
    pub(super) bodies: Vec<ControlBody>,
    /// Name of an untyped counting `for` variable, after which `: integer` goes.
    pub(super) for_variable: Option<Span>,
    /// An untyped `for … in` variable, whose element type is unknown.
    pub(super) untyped_for_in: bool,
    /// Starts of Pascal case arms written without `when`.
    pub(super) arm_starts: Vec<Span>,
}

/// One parsed branch or loop body: its statement count and first statement.
#[derive(Debug)]
pub(super) struct ControlBody {
    pub(super) statements: usize,
    pub(super) first: Option<Span>,
}

impl Parser {
    /// Notes a branch or loop body that starts with `begin`.
    pub(in crate::parser) fn note_compound_body(&self, control: &mut PascalControl) {
        if self.check(&Token::Begin) {
            control.compound_body = true;
        }
    }

    /// Notes the missing `;` error just reported before `else`, if any.
    pub(in crate::parser) fn note_else_terminator(
        &self,
        control: &mut PascalControl,
        errors_before: usize,
    ) {
        if !self.check(&Token::Else) {
            return;
        }
        let else_offset = self.current_span().offset;
        control.else_terminator_error = (errors_before..self.errors.len()).rev().find(|&index| {
            let error = &self.errors[index];
            error.found.as_deref() == Some("else")
                && error.expected.as_deref() == Some(";")
                && error.span.is_some_and(|span| span.offset() == else_offset)
        });
    }

    /// Notes Pascal `for I := …` or `for I in …` without a type; recovery continues with
    /// `integer`. `variable` is the loop variable's name.
    pub(in crate::parser) fn note_untyped_for(&self, control: &mut PascalControl, variable: Span) {
        control.untyped_for = true;
        if self.check(&Token::In) {
            control.untyped_for_in = true;
        } else {
            control.for_variable = Some(variable);
        }
    }

    /// Notes a Pascal case arm without `when` that starts at `label`.
    pub(in crate::parser) fn note_arm_without_when(
        &self,
        control: &mut PascalControl,
        label: Span,
    ) {
        control.arms_without_when = true;
        control.arm_starts.push(label);
    }

    /// Notes a parsed branch or loop body.
    pub(in crate::parser) fn note_body(&self, control: &mut PascalControl, body: &Stmt) {
        if let Stmt::Block(statements, _) = body {
            control.bodies.push(ControlBody {
                statements: statements.len(),
                first: statements.first().map(Stmt::span),
            });
        }
    }

    /// Requires the named ending of a control statement and collapses its Pascal follow-up
    /// errors into one diagnostic. `form` is the FPAS form shown in the help.
    pub(in crate::parser) fn finish_pascal_control(
        &mut self,
        kind: &Token,
        start: Span,
        control: PascalControl,
        form: &str,
        nested_else_if: bool,
    ) {
        let errors_before = self.errors.len();
        let end_token = self.check(&Token::End).then(|| self.current_span());
        let pascal = control.compound_body
            || control.else_terminator_error.is_some()
            || control.untyped_for
            || control.arms_without_when;
        // Pascal `if`, `while` and `for` have no ending of their own, so a following `end;` or
        // `end.` belongs to the enclosing routine or program. A Pascal `case` owns its `end;`.
        let ending_missing = if pascal
            && kind != &Token::Case
            && self.check(&Token::End)
            && matches!(self.peek_token(), Token::Semicolon | Token::Dot)
        {
            true
        } else {
            if kind == &Token::If {
                self.expect_if_end(nested_else_if);
            } else {
                self.expect_block_end(kind);
            }
            self.errors.len() > errors_before
        };
        if !pascal || (!ending_missing && !control.untyped_for && !control.arms_without_when) {
            return;
        }
        self.errors.truncate(errors_before);
        if let Some(index) = control.else_terminator_error {
            self.errors.remove(index);
        }
        let keyword = token_display(kind);
        let mut explanation = Vec::new();
        if control.untyped_for {
            explanation.push("The loop declares its variable with a type.".to_owned());
        }
        if control.arms_without_when {
            explanation.push("Every case arm starts with `when`.".to_owned());
        }
        if control.compound_body {
            explanation.push(
                "`begin … end` inside a branch or loop is a nested block, not its body.".to_owned(),
            );
        }
        if control.else_terminator_error.is_some() {
            explanation.push("Every statement ends with `;`, also before `else`.".to_owned());
        }
        if ending_missing {
            explanation.push(format!("The statement closes with `end {keyword};`."));
        }
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        let fix = self.pascal_control_fix(kind, &control, written, ending_missing, end_token);
        let index = self.report_foreign_habit(
            PARSE_EXPECTED_TOKEN,
            &format!("Pascal `{keyword}` statement is not FPAS syntax"),
            ForeignHabit {
                found: &found,
                replacement: form,
                explanation: &explanation.join(" "),
            },
            written,
            None,
        );
        self.add_habit_fix(index, fix);
    }

    /// True at Ada `loop` where FPAS expects `do`.
    pub(in crate::parser) fn at_ada_loop(&self) -> bool {
        matches!(self.current_token(), Token::Ident(word) if word.eq_ignore_ascii_case("loop"))
    }

    /// Skips an Ada loop body through `end loop;` and reports the loop once.
    pub(in crate::parser) fn skip_ada_loop(&mut self, start: Span, form: &str) -> Stmt {
        self.advance();
        let mut depth = 0_usize;
        while !self.at_end() {
            if self.check(&Token::End) {
                if matches!(self.peek_token(), Token::Ident(word) if word.eq_ignore_ascii_case("loop"))
                {
                    self.advance();
                    self.advance();
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    continue;
                }
                if self.peek_token() == &Token::Dot || self.at_enclosing_block_end() {
                    break;
                }
            } else if self.at_ada_loop() {
                depth += 1;
            }
            self.advance();
        }
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            "Ada loop is not FPAS syntax",
            ForeignHabit {
                found: &found,
                replacement: form,
                explanation: "FPAS loops use `do`, declare a typed loop variable, and close with a named ending.",
            },
            written,
            None,
        );
        Stmt::Block(Vec::new(), written)
    }

    /// True at a Delphi `try` statement: `try` followed by statements and `except` or
    /// `finally`, rather than the FPAS `try Expr` propagation.
    pub(in crate::parser) fn at_delphi_try(&self) -> bool {
        self.check(&Token::Try) && self.delphi_try_handler().is_some()
    }

    /// Skips a Delphi `try … except/finally … end;` and reports it once.
    pub(in crate::parser) fn skip_delphi_try(&mut self) -> Stmt {
        let handler = self.delphi_try_handler().unwrap_or("except");
        let start = self.advance().span;
        let end = self.delphi_try_end(self.pos);
        while self.pos < end && !self.at_end() {
            self.advance();
        }
        let written = self.span_from(start);
        let found = self.source_text(written).to_owned();
        let (replacement, explanation) = if handler == "finally" {
            (
                "Work(); Cleanup();",
                "FPAS has no exceptions or `finally`; failures are values, so cleanup runs as ordinary statements after the work.",
            )
        } else {
            (
                "case Operation() of when Ok(const Value): …; when Error(const Message): …; end case;",
                "FPAS has no exception handlers; return `Result of (T, E)` and handle failures with `case`, or propagate them with `try Expr`.",
            )
        };
        self.report_foreign_habit(
            PARSE_FOREIGN_SYNTAX,
            &format!("Delphi `try … {handler}` is not FPAS syntax"),
            ForeignHabit {
                found: &found,
                replacement,
                explanation,
            },
            written,
            None,
        );
        Stmt::Block(Vec::new(), written)
    }

    /// Finds `except` or `finally` that belongs to the `try` at the current token.
    fn delphi_try_handler(&self) -> Option<&'static str> {
        let end = self.delphi_try_end(self.pos + 1);
        self.tokens[self.pos + 1..end.min(self.tokens.len())]
            .iter()
            .find_map(|token| match &token.token {
                Token::Ident(word) if word.eq_ignore_ascii_case("except") => Some("except"),
                Token::Ident(word) if word.eq_ignore_ascii_case("finally") => Some("finally"),
                _ => None,
            })
            .filter(|_| end < self.tokens.len())
    }

    /// Returns the token index just after the `end` that closes a Delphi block starting at
    /// `from`, or the token count when none closes it within the scan limit. FPAS named endings
    /// such as `end if` close their own constructs and are skipped.
    fn delphi_try_end(&self, from: usize) -> usize {
        let mut depth = 0_usize;
        let limit = (from + MAX_TRY_SCAN).min(self.tokens.len());
        let mut index = from;
        while index < limit {
            let next = self.tokens.get(index + 1).map(|token| &token.token);
            match &self.tokens[index].token {
                Token::Begin | Token::Try | Token::Case => depth += 1,
                Token::End if next.is_some_and(is_named_fpas_ending) => index += 1,
                Token::End if depth == 0 => return index + 1,
                Token::End => depth -= 1,
                Token::Eof => break,
                _ => {}
            }
            index += 1;
        }
        self.tokens.len()
    }
}

/// Keywords after `end` that close an FPAS construct other than a Delphi block.
fn is_named_fpas_ending(token: &Token) -> bool {
    matches!(
        token,
        Token::If
            | Token::For
            | Token::While
            | Token::Function
            | Token::Procedure
            | Token::Record
            | Token::Enum
            | Token::Scope
            | Token::With
    )
}

impl Parser {
    /// True at `of Label:` or `of Label,` where a Pascal case arm without `when` follows the
    /// selector, so the selector is not read as a type application.
    pub(in crate::parser) fn at_pascal_case_label_after_of(&self) -> bool {
        self.check(&Token::Of)
            && matches!(self.peek_token(), Token::Ident(_))
            && matches!(
                self.tokens.get(self.pos + 2).map(|token| &token.token),
                Some(Token::Colon | Token::Comma)
            )
    }
}
