//! Case label dispatch: scalar values and ranges, scalar bindings, and patterns.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/README.md`

use super::Checker;
use super::coverage::Pat;
use super::patterns::PatternBindings;
use crate::types::{EnumTy, Ty};
use fpas_diagnostics::codes::{SEMA_NON_BOOLEAN_CONDITION, SEMA_TYPE_MISMATCH};
use fpas_lexer::Span;
use fpas_parser::{CaseLabel, DestructureVariant, Expr, Pattern};

/// Result of checking one label: the arm bindings it introduces and its coverage view.
pub(super) struct CheckedLabel {
    pub(super) bindings: PatternBindings,
    pub(super) pat: Pat,
}

impl Checker {
    /// Checks one label against the case expression type.
    pub(super) fn check_case_label(
        &mut self,
        case_ty: &Ty,
        is_pattern_case: bool,
        label: &CaseLabel,
    ) -> CheckedLabel {
        match label {
            CaseLabel::Value { start, end, span } => {
                if is_pattern_case && self.resolve_enum_ty(case_ty).is_none_or(|e| e.has_data()) {
                    if end.is_some() {
                        self.error_with_code(
                            SEMA_TYPE_MISMATCH,
                            "Pattern case labels do not support ranges",
                            "Match variants directly, for example `Shape.Circle(const R)` or `Shape.Point`.",
                            *span,
                        );
                        return CheckedLabel::wild();
                    }
                    let mut bindings = PatternBindings::new();
                    let pat = self.check_value_pattern(case_ty, start, &mut bindings);
                    return CheckedLabel { bindings, pat };
                }

                if self.reject_unknown_scalar_label(start) {
                    return CheckedLabel::wild();
                }
                let label_ty = self.check_expr(start);
                self.check_type_compat(case_ty, &label_ty, "case label", *span);
                if !label_ty.is_error() {
                    self.require_case_constant(start);
                }
                if let Some(range_end) = end {
                    let end_ty = self.check_expr(range_end);
                    self.check_type_compat(case_ty, &end_ty, "case label range end", *span);
                    if !end_ty.is_error() {
                        self.require_case_constant(range_end);
                    }
                    return CheckedLabel::other();
                }
                let pat = if label_ty.is_error() {
                    Pat::Wild
                } else {
                    self.value_constructor(case_ty, start)
                };
                CheckedLabel {
                    bindings: PatternBindings::new(),
                    pat,
                }
            }
            CaseLabel::Pattern(Pattern::Variant {
                constructor, span, ..
            }) if !is_pattern_case => {
                // In a scalar case, `Name(...)` is a call, which is never a constant.
                self.require_case_constant_call(constructor, *span);
                CheckedLabel::wild()
            }
            CaseLabel::Pattern(pattern) => {
                let (bindings, pat) = self.check_label_pattern(case_ty, pattern);
                CheckedLabel { bindings, pat }
            }
            // `scalar_case_binding` validates the arm before its labels are checked.
            CaseLabel::Binding { .. } => CheckedLabel::wild(),
        }
    }

    pub(super) fn check_guard(&mut self, guard: &Option<Expr>, span: Span) {
        if let Some(guard_expr) = guard {
            let guard_ty = self.check_expr(guard_expr);
            if matches!(guard_ty, Ty::GenericParam(..)) {
                self.check_type_compat(&Ty::Boolean, &guard_ty, "case guard", span);
            } else if !Ty::Boolean.assignment_compatible_with(&guard_ty) {
                self.error_with_code(
                    SEMA_NON_BOOLEAN_CONDITION,
                    "Guard clause must be a boolean expression",
                    "when Label if <boolean>: ...",
                    span,
                );
            }
        }
    }

    pub(super) fn resolve_enum_ty<'a>(&'a self, ty: &'a Ty) -> Option<&'a EnumTy> {
        match ty {
            Ty::Enum(enum_ty) => Some(enum_ty),
            Ty::Named(name) => {
                let sym = self.scopes.lookup(name)?;
                match &sym.ty {
                    Ty::Enum(enum_ty) => Some(enum_ty),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

impl CheckedLabel {
    fn wild() -> Self {
        Self {
            bindings: PatternBindings::new(),
            pat: Pat::Wild,
        }
    }

    fn other() -> Self {
        Self {
            bindings: PatternBindings::new(),
            pat: Pat::Other,
        }
    }
}

pub(super) fn binding_type_for_variant(case_ty: &Ty, variant: &DestructureVariant) -> Ty {
    match (case_ty, variant) {
        (Ty::Result(ok, _), DestructureVariant::Ok) => *ok.clone(),
        (Ty::Result(_, err), DestructureVariant::Error) => *err.clone(),
        (Ty::Option(inner), DestructureVariant::Some) => *inner.clone(),
        _ => Ty::Error,
    }
}
