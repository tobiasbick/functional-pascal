//! Exhaustiveness and usefulness of recursively typed pattern rows.

mod rows;
mod usefulness;

use super::{Checker, PatternVariant};
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_NON_EXHAUSTIVE_CASE, SEMA_TYPE_MISMATCH};
use fpas_lexer::Span;
use fpas_parser::{CaseArm, Pattern};

#[derive(Clone, Debug, PartialEq, Eq)]
enum Tag {
    Variant(PatternVariant),
    Boolean(bool),
    Integer(i64, i64),
    String(String, String),
    Real(u64),
    Opaque(String),
}

#[derive(Clone, Debug)]
enum RowPattern {
    Any,
    Constructor(Tag, Vec<RowPattern>),
}

impl Checker {
    /// Check usefulness and required coverage, returning whether all values are covered.
    pub(in crate::check) fn check_recursive_case_coverage<Body>(
        &mut self,
        ty: &Ty,
        arms: &[CaseArm<Body>],
        has_else: bool,
        require_value: bool,
        span: Span,
    ) -> bool {
        let ty = self.resolve_visible_type(ty);
        let closed = matches!(ty, Ty::Enum(_) | Ty::Option(_) | Ty::Result(..));
        if closed && has_else {
            self.error_with_code(SEMA_TYPE_MISMATCH, "Closed enum cases cannot have an else branch",
                "Name every variant explicitly; use `_` within a variant for remaining payload values.", span);
        }
        let mut matrix = Vec::new();
        for arm in arms {
            let mut alternatives = matrix.clone();
            for label in &arm.labels {
                if matches!(label, Pattern::Wildcard(_))
                    || (closed && matches!(label, Pattern::Binding { .. }))
                {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        "A closed case must name its top-level variant; `_` is a payload wildcard",
                        "Write a qualified variant such as `Choice.Present(_)`.",
                        arm.span,
                    );
                }
                let row = vec![self.coverage_pattern(label)];
                if closed && !matches!(&row[0], RowPattern::Constructor(Tag::Variant(_), _)) {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        "Closed enum cases require explicitly named variants",
                        "Match the qualified variant and then its payload patterns.",
                        arm.span,
                    );
                }
                if !self.pattern_row_useful(&alternatives, &row, std::slice::from_ref(&ty)) {
                    self.error_with_code(SEMA_TYPE_MISMATCH, "Case pattern is duplicate or unreachable",
                        "Remove the covered pattern or put a guarded, more specific arm before its fallback.", arm.span);
                }
                alternatives.push(row.clone());
                if arm.guard.is_none() {
                    matrix.push(row);
                }
            }
        }
        let incomplete = self.pattern_row_useful(&matrix, &[RowPattern::Any], &[ty]);
        if incomplete && (closed || (require_value && !has_else)) {
            self.error_with_code(SEMA_NON_EXHAUSTIVE_CASE, "Non-exhaustive case: some variants or payload values are missing",
                "Cover nested alternatives explicitly or use payload `_`; guards do not establish coverage. Open scalar value cases require else.", span);
        }
        if !incomplete && has_else && !closed {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Case else branch is unreachable",
                "Remove else after complete finite coverage.",
                span,
            );
        }
        !incomplete
    }
}
