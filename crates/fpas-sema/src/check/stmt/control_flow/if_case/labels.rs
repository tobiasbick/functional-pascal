use super::Checker;
use crate::types::Ty;
use fpas_diagnostics::codes::{SEMA_NON_BOOLEAN_CONDITION, SEMA_TYPE_MISMATCH};
use fpas_lexer::Span;
use fpas_parser::{CaseLabel, DesignatorPart, DestructureVariant, Expr};

impl Checker {
    pub(super) fn check_case_label(
        &mut self,
        case_ty: &Ty,
        is_result_or_option: bool,
        is_data_enum: bool,
        label: &CaseLabel,
    ) -> Option<Vec<(String, Ty)>> {
        match label {
            CaseLabel::Value { start, end, span } => {
                if is_data_enum {
                    return Some(self.extract_enum_pattern_bindings(case_ty, start));
                }

                let label_ty = self.check_expr(start);
                self.check_type_compat(case_ty, &label_ty, "case label", *span);
                if let Some(range_end) = end {
                    let end_ty = self.check_expr(range_end);
                    self.check_type_compat(case_ty, &end_ty, "case label range end", *span);
                }
                None
            }
            CaseLabel::Destructure {
                variant,
                binding,
                span,
            } => {
                if !is_result_or_option {
                    self.error_with_code(
                        SEMA_TYPE_MISMATCH,
                        "Destructure patterns (Ok/Err/Some/None) require Result or Option case expression",
                        "Use destructure patterns only with Result or Option values.",
                        *span,
                    );
                }

                binding.as_ref().map(|binding_name| {
                    let binding_ty = binding_type_for_variant(case_ty, variant);
                    vec![(binding_name.clone(), binding_ty)]
                })
            }
        }
    }

    pub(super) fn check_guard(&mut self, guard: &Option<Expr>, span: Span) {
        if let Some(guard_expr) = guard {
            let guard_ty = self.check_expr(guard_expr);
            if !guard_ty.compatible_with(&Ty::Boolean) {
                self.error_with_code(
                    SEMA_NON_BOOLEAN_CONDITION,
                    "Guard clause must be a boolean expression",
                    "case Label if <boolean>: ...",
                    span,
                );
            }
        }
    }

    /// Extract binding names and their types from an enum variant pattern in a
    /// `case` label. Supports nested patterns like `Expr.Add(Expr.Num(A), Expr.Num(B))`.
    ///
    /// **Documentation:** `docs/pascal/06-pattern-matching.md`
    pub(super) fn extract_enum_pattern_bindings(
        &self,
        case_ty: &Ty,
        expr: &Expr,
    ) -> Vec<(String, Ty)> {
        let enum_ty = match case_ty {
            Ty::Enum(enum_ty) => enum_ty,
            Ty::Named(name) => {
                let Some(sym) = self.scopes.lookup(name) else {
                    return Vec::new();
                };
                let Ty::Enum(enum_ty) = &sym.ty else {
                    return Vec::new();
                };
                enum_ty
            }
            _ => return Vec::new(),
        };

        match expr {
            Expr::Call {
                designator, args, ..
            } => {
                let variant_name = designator.parts.last().and_then(|part| match part {
                    DesignatorPart::Ident(name, _) => Some(name.as_str()),
                    _ => None,
                });
                let Some(variant_name) = variant_name else {
                    return Vec::new();
                };
                let Some(variant) = enum_ty
                    .variants
                    .iter()
                    .find(|variant| variant.name == variant_name)
                else {
                    return Vec::new();
                };

                let mut bindings = Vec::new();
                for (arg, (_, field_ty)) in args.iter().zip(variant.fields.iter()) {
                    match arg {
                        Expr::Designator(designator) if designator.parts.len() == 1 => {
                            if let DesignatorPart::Ident(name, _) = &designator.parts[0] {
                                if name != "_" {
                                    bindings.push((name.clone(), field_ty.clone()));
                                }
                            }
                        }
                        Expr::Call { .. } => {
                            bindings.extend(self.extract_enum_pattern_bindings(field_ty, arg));
                        }
                        Expr::Designator(_) => {}
                        _ => {}
                    }
                }
                bindings
            }
            Expr::Designator(_) => Vec::new(),
            _ => Vec::new(),
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
