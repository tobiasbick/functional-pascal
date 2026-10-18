//! Recognition of spellings from other languages (Delphi, Free Pascal, Ada, C family) that the
//! parser rejects with one diagnostic showing the FPAS form, followed by recovery as if the FPAS
//! form had been written.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

mod aggregates;
mod arguments;
mod bindings;
mod classes;
mod control_flow;
mod control_flow_fix;
mod generics;
mod programs;
mod routines;
mod types;
mod units;

pub(in crate::parser) use bindings::{BindingContext, BindingKeyword};
pub(in crate::parser) use control_flow::PascalControl;
pub(in crate::parser) use routines::AdaHeaderParts;

use super::Parser;
use crate::error::parse_error;
use fpas_diagnostics::{Applicability, DiagnosticCode, Fix, ForeignHabit, SourceSpan};
use fpas_lexer::Span;

impl Parser {
    /// Reports one foreign-habit diagnostic; `span` must cover exactly `habit.found`. Returns
    /// the diagnostic's index, or `None` when the nesting limit suppressed it.
    ///
    /// With `fix`, the diagnostic carries an edit that replaces `span` with `habit.replacement`;
    /// pass it only when the replacement is complete FPAS text without placeholders.
    pub(in crate::parser) fn report_foreign_habit(
        &mut self,
        code: DiagnosticCode,
        message: &str,
        habit: ForeignHabit<'_>,
        span: Span,
        fix: Option<Applicability>,
    ) -> Option<usize> {
        if self.nesting_limit_reached {
            return None;
        }
        let mut diagnostic = habit.attach(parse_error(code, message, String::new(), span));
        if let Some(applicability) = fix {
            diagnostic = diagnostic.with_fix(habit.fix(
                applicability,
                SourceSpan::try_from(span).ok(),
                habit.replacement,
            ));
        }
        self.errors.push(diagnostic);
        Some(self.errors.len() - 1)
    }

    /// Adds `fix` to the habit diagnostic at `index`, as returned by [`Self::report_foreign_habit`].
    pub(in crate::parser) fn add_habit_fix(&mut self, index: Option<usize>, fix: Option<Fix>) {
        if let Some(diagnostic) = index.and_then(|index| self.errors.get_mut(index)) {
            diagnostic.fixes.extend(fix);
        }
    }

    /// Returns the source text covered by `span`, or an empty string for synthetic spans.
    pub(in crate::parser) fn source_text(&self, span: Span) -> &str {
        self.source
            .get(span.offset..span.offset + span.length)
            .unwrap_or_default()
    }
}

impl Parser {
    /// A named type placeholder used when recovery has to supply a type the author omitted.
    pub(in crate::parser) fn named_type(name: &str, span: Span) -> crate::ast::TypeExpr {
        crate::ast::TypeExpr::Named {
            id: crate::ast::QualifiedId {
                parts: vec![name.to_owned()],
                span,
            },
            span,
        }
    }
}
