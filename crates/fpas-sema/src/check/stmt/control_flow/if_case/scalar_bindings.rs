//! Scalar guard bindings (`when const N if Guard:`) and bare scalar labels.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/guards.md`

use super::Checker;
use fpas_diagnostics::codes::{
    SEMA_IMPLICIT_PATTERN_BINDING, SEMA_INVALID_CASE_BINDING, SEMA_TYPE_MISMATCH,
};
use fpas_parser::{CaseLabel, DesignatorPart, Expr};

impl Checker {
    /// Validates `when const Name if Guard:` and returns the bound name when the arm uses it.
    pub(super) fn scalar_case_binding<'a>(
        &mut self,
        is_scalar_case: bool,
        labels: &'a [CaseLabel],
        guard: &Option<Expr>,
    ) -> Option<&'a str> {
        let Some((name, span)) = labels.iter().find_map(|label| match label {
            CaseLabel::Binding { name, span } => Some((name.as_str(), *span)),
            _ => None,
        }) else {
            // Recover a former implicit binding as a binding after reporting it.
            return match labels {
                [
                    CaseLabel::Value {
                        start, end: None, ..
                    },
                ] if is_scalar_case
                    && guard.is_some()
                    && self.reject_unknown_scalar_label(start) =>
                {
                    match start {
                        Expr::Designator(designator) => match designator.parts.as_slice() {
                            [DesignatorPart::Ident(name, _)] if name != "_" => Some(name.as_str()),
                            _ => None,
                        },
                        _ => None,
                    }
                }
                _ => None,
            };
        };
        let problem = if !is_scalar_case {
            Some((
                "`const` labels bind scalar case values only".to_string(),
                format!(
                    "Bind payloads inside their pattern, for example `Some(const {name})` or `Shape.Circle(const {name})`."
                ),
            ))
        } else if labels.len() != 1 {
            Some((
                format!("`const {name}` must be the only label of its case arm"),
                "Move other labels to a separate arm.".to_string(),
            ))
        } else if guard.is_none() {
            Some((
                format!("`const {name}` requires a guard"),
                format!("Add a condition, for example `when const {name} if {name} > 0:`."),
            ))
        } else {
            None
        };
        if let Some((message, hint)) = problem {
            self.error_with_code(SEMA_INVALID_CASE_BINDING, message, hint, span);
        }
        Some(name)
    }

    /// Reports a bare label that names no value, showing the explicit binding form.
    pub(super) fn reject_unknown_scalar_label(&mut self, start: &Expr) -> bool {
        let Expr::Designator(designator) = start else {
            return false;
        };
        let [DesignatorPart::Ident(name, _)] = designator.parts.as_slice() else {
            return false;
        };
        if name == "_" {
            self.reject_whole_value_wildcard(designator.span);
            return true;
        }
        if self.scopes.lookup(name).is_some() {
            return false;
        }
        self.error_with_code(
            SEMA_IMPLICIT_PATTERN_BINDING,
            format!("Case label `{name}` is not a constant"),
            format!(
                "To bind the matched value, write `when const {name} if Guard:`; otherwise use a literal or a compile-time constant."
            ),
            designator.span,
        );
        true
    }

    /// `_` ignores one payload field; it never stands for a whole case value.
    pub(in crate::check::stmt::control_flow) fn reject_whole_value_wildcard(
        &mut self,
        span: fpas_lexer::Span,
    ) {
        self.error_with_code(
            SEMA_TYPE_MISMATCH,
            "`_` cannot stand for a whole case value",
            "`_` ignores one payload field, for example `Some(_)`. List remaining variants explicitly for closed enums, or use `else` for scalar cases.",
            span,
        );
    }
}
