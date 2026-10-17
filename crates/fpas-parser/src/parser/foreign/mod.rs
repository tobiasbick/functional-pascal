//! Recognition of spellings from other languages (Delphi, Free Pascal, Ada, C family) that the
//! parser rejects with one diagnostic showing the FPAS form, followed by recovery as if the FPAS
//! form had been written.
//!
//! **Documentation:** `docs/pascal/tools/diagnostics.md` (Diagnostics for other-language spellings)

mod routines;

pub(in crate::parser) use routines::AdaHeaderParts;

use super::Parser;
use crate::error::parse_error;
use fpas_diagnostics::{DiagnosticCode, ForeignHabit};
use fpas_lexer::Span;

impl Parser {
    /// Reports one foreign-habit diagnostic; `span` must cover exactly `habit.found`.
    pub(in crate::parser) fn report_foreign_habit(
        &mut self,
        code: DiagnosticCode,
        message: &str,
        habit: ForeignHabit<'_>,
        span: Span,
    ) {
        if self.nesting_limit_reached {
            return;
        }
        let diagnostic = habit.attach(parse_error(code, message, String::new(), span));
        self.errors.push(diagnostic);
    }

    /// Returns the source text covered by `span`, or an empty string for synthetic spans.
    pub(in crate::parser) fn source_text(&self, span: Span) -> &str {
        self.source
            .get(span.offset..span.offset + span.length)
            .unwrap_or_default()
    }
}
