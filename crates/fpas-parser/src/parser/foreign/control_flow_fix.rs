//! The rewrite fix for a Pascal control statement: `begin … end` bodies unwrapped, `;` before
//! `else`, typed `for` variables, `when` before case arms, and the named ending.
//!
//! A Pascal branch or loop body is exactly one statement, so the named ending goes directly
//! after the first statement of the last body; statements after it stay outside, as in Pascal.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Fixes)

use super::super::{Parser, token_display};
use super::control_flow::PascalControl;
use fpas_diagnostics::{Applicability, Fix, FixEdit, SourceSpan};
use fpas_lexer::{Span, Token};

impl Parser {
    /// Builds the fix for a Pascal control statement written over `written`, or `None` when
    /// the rewrite is not certain: an untyped `for … in`, other errors inside the statement,
    /// or a branch with more than one statement before `else`.
    pub(super) fn pascal_control_fix(
        &self,
        kind: &Token,
        control: &PascalControl,
        written: Span,
        ending_missing: bool,
        end_token: Option<Span>,
    ) -> Option<Fix> {
        if control.untyped_for_in || self.has_error_within(written) {
            return None;
        }
        let keyword = token_display(kind);
        let mut edits = Vec::new();
        if let Some(variable) = control.for_variable {
            edits.push(insert_at(self.end_of(variable), ": integer")?);
        }
        for &arm in &control.arm_starts {
            edits.push(insert_at(arm, "when ")?);
        }
        if ending_missing && kind == &Token::Case {
            edits.push(insert_at(self.end_of(end_token?), &format!(" {keyword}"))?);
        } else if ending_missing {
            let last = control.bodies.len().checked_sub(1)?;
            for (index, body) in control.bodies.iter().enumerate() {
                let first = body.first?;
                let is_last = index == last;
                if !is_last && body.statements != 1 {
                    return None;
                }
                if let Some((begin, end)) = self.compound_tokens(first) {
                    edits.push(self.deletion(begin)?);
                    edits.push(if is_last {
                        replace(end, &format!("end {keyword}"))?
                    } else {
                        self.deletion(end)?
                    });
                } else if is_last {
                    edits.push(self.ending_after(first, written, &keyword)?);
                } else if control.else_terminator_error.is_some() {
                    edits.push(insert_at(self.end_of(first), ";")?);
                }
            }
        }
        Fix::new(
            format!("Rewrite as an FPAS `{keyword}` statement"),
            Applicability::MachineApplicable,
            edits,
        )
    }

    /// True when a diagnostic other than the statement's own lies inside `written`.
    fn has_error_within(&self, written: Span) -> bool {
        let end = written.offset + written.length;
        self.errors.iter().any(|error| {
            error
                .span
                .is_some_and(|span| span.offset() >= written.offset && span.offset() < end)
        })
    }

    /// Returns the `begin` and `end` tokens when the statement at `statement` is a
    /// `begin … end` block.
    fn compound_tokens(&self, statement: Span) -> Option<(Span, Span)> {
        let first = self.token_index_at(statement.offset)?;
        if self.tokens[first].token != Token::Begin {
            return None;
        }
        let statement_end = statement.offset + statement.length;
        let last = self
            .tokens
            .partition_point(|token| token.span.offset < statement_end)
            .checked_sub(1)?;
        let end = &self.tokens[last];
        (end.token == Token::End && end.span.offset + end.span.length == statement_end)
            .then_some((self.tokens[first].span, end.span))
    }

    /// Inserts `end KEYWORD;` on its own line after the statement at `statement` and its `;`,
    /// indented like the control statement over `written`.
    fn ending_after(&self, statement: Span, written: Span, keyword: &str) -> Option<FixEdit> {
        let source = self.source.as_str();
        let (line_start, _) = self.line_bounds(written.offset, written.offset);
        let indent: String = source[line_start..written.offset]
            .chars()
            .take_while(|character| matches!(character, ' ' | '\t'))
            .collect();
        let newline = if source.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let ending = format!("{newline}{indent}end {keyword};");
        let statement_end = statement.offset + statement.length;
        let next = self.tokens.get(
            self.tokens
                .partition_point(|token| token.span.offset < statement_end),
        )?;
        if next.token == Token::Semicolon {
            insert_at(self.end_of(next.span), &ending)
        } else {
            insert_at(self.end_of(statement), &format!(";{ending}"))
        }
    }

    /// The zero-length span just after `span`, with its line and column.
    fn end_of(&self, span: Span) -> Span {
        let written = self.source_text(span);
        let (line, column) = match written.rfind('\n') {
            Some(newline) => (
                span.line + u32::try_from(written.matches('\n').count()).unwrap_or(0),
                1 + u32::try_from(written[newline + 1..].chars().count()).unwrap_or(0),
            ),
            None => (
                span.line,
                span.column + u32::try_from(written.chars().count()).unwrap_or(0),
            ),
        };
        Span {
            offset: span.offset + span.length,
            length: 0,
            line,
            column,
            source_id: span.source_id,
        }
    }

    /// Deletes the token at `span`, with its whole line when nothing else is on it, or with
    /// the spaces before it when it ends its line.
    fn deletion(&self, span: Span) -> Option<FixEdit> {
        let source = self.source.as_str();
        let end = span.offset + span.length;
        let (line_start, line_end) = self.line_bounds(span.offset, end);
        let before = &source[line_start..span.offset];
        if !source[end..line_end].trim().is_empty() {
            return replace(span, "");
        }
        let (start, column, stop) = if before.trim().is_empty() {
            (line_start, 1, (line_end + 1).min(source.len()))
        } else {
            let spaces = before.len() - before.trim_end().len();
            let spaces_column = u32::try_from(spaces).unwrap_or(0);
            let column = span.column.saturating_sub(spaces_column).max(1);
            (span.offset - spaces, column, line_end)
        };
        replace(
            Span {
                offset: start,
                length: stop - start,
                column,
                ..span
            },
            "",
        )
    }

    /// Returns the start of the line containing `start` and the offset of the line break
    /// after `end`, or the source length.
    fn line_bounds(&self, start: usize, end: usize) -> (usize, usize) {
        let source = self.source.as_str();
        let line_start = source[..start].rfind('\n').map_or(0, |index| index + 1);
        let line_end = source[end..]
            .find('\n')
            .map_or(source.len(), |index| end + index);
        (line_start, line_end)
    }

    fn token_index_at(&self, offset: usize) -> Option<usize> {
        let index = self
            .tokens
            .partition_point(|token| token.span.offset < offset);
        (self.tokens.get(index)?.span.offset == offset).then_some(index)
    }
}

fn insert_at(span: Span, text: &str) -> Option<FixEdit> {
    replace(Span { length: 0, ..span }, text)
}

fn replace(span: Span, text: &str) -> Option<FixEdit> {
    Some(FixEdit {
        span: SourceSpan::try_from(span).ok()?,
        new_text: text.to_owned(),
    })
}
