use super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_NON_EXHAUSTIVE_CASE;
use fpas_lexer::Span;
use fpas_parser::{CaseArm, CaseLabel, DesignatorPart, DestructureVariant, Expr};

impl Checker {
    /// Check that a `case` statement covers all variants of an enum, Result,
    /// or Option type. Arms with guard clauses do not count toward coverage.
    ///
    /// **Documentation:** `docs/pascal/06-pattern-matching.md`
    pub(super) fn check_exhaustiveness(&mut self, case_ty: &Ty, arms: &[CaseArm], span: Span) {
        match case_ty {
            Ty::Enum(enum_ty) => {
                let covered: Vec<&str> = arms
                    .iter()
                    .filter(|arm| arm.guard.is_none())
                    .flat_map(|arm| &arm.labels)
                    .filter_map(|label| match label {
                        CaseLabel::Value { start, .. } => variant_name_from_expr(start),
                        _ => None,
                    })
                    .collect();
                let missing: Vec<&str> = enum_ty
                    .variants
                    .iter()
                    .filter(|variant| {
                        !covered
                            .iter()
                            .any(|covered_name| covered_name.eq_ignore_ascii_case(&variant.name))
                    })
                    .map(|variant| variant.name.as_str())
                    .collect();
                if !missing.is_empty() {
                    let list = missing.join(", ");
                    self.error_with_code(
                        SEMA_NON_EXHAUSTIVE_CASE,
                        format!("Non-exhaustive case: missing variant(s) {list}"),
                        "Cover all variants or add an else branch.",
                        span,
                    );
                }
            }
            Ty::Result(_, _) => {
                let (mut has_ok, mut has_err) = (false, false);
                for arm in arms.iter().filter(|arm| arm.guard.is_none()) {
                    for label in &arm.labels {
                        if let CaseLabel::Destructure { variant, .. } = label {
                            match variant {
                                DestructureVariant::Ok => has_ok = true,
                                DestructureVariant::Error => has_err = true,
                                _ => {}
                            }
                        }
                    }
                }

                let mut missing = Vec::new();
                if !has_ok {
                    missing.push("Ok");
                }
                if !has_err {
                    missing.push("Error");
                }
                if !missing.is_empty() {
                    let list = missing.join(", ");
                    self.error_with_code(
                        SEMA_NON_EXHAUSTIVE_CASE,
                        format!("Non-exhaustive case: missing {list}"),
                        "Cover both Ok and Error or add an else branch.",
                        span,
                    );
                }
            }
            Ty::Option(_) => {
                let (mut has_some, mut has_none) = (false, false);
                for arm in arms.iter().filter(|arm| arm.guard.is_none()) {
                    for label in &arm.labels {
                        if let CaseLabel::Destructure { variant, .. } = label {
                            match variant {
                                DestructureVariant::Some => has_some = true,
                                DestructureVariant::None => has_none = true,
                                _ => {}
                            }
                        }
                    }
                }

                let mut missing = Vec::new();
                if !has_some {
                    missing.push("Some");
                }
                if !has_none {
                    missing.push("None");
                }
                if !missing.is_empty() {
                    let list = missing.join(", ");
                    self.error_with_code(
                        SEMA_NON_EXHAUSTIVE_CASE,
                        format!("Non-exhaustive case: missing {list}"),
                        "Cover both Some and None or add an else branch.",
                        span,
                    );
                }
            }
            _ => {}
        }
    }
}

fn variant_name_from_expr(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Call { designator, .. } | Expr::Designator(designator) => {
            designator.parts.last().and_then(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                _ => None,
            })
        }
        _ => None,
    }
}
