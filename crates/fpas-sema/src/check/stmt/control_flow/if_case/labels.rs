use super::Checker;
use crate::check::spans::expr_span;
use crate::types::{EnumTy, Ty};
use fpas_diagnostics::codes::{
    SEMA_ENUM_FIELD_COUNT_MISMATCH, SEMA_NON_BOOLEAN_CONDITION, SEMA_TYPE_MISMATCH,
};
use fpas_lexer::Span;
use fpas_parser::{CaseLabel, Designator, DesignatorPart, DestructureVariant, Expr};

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
                    if end.is_some() {
                        self.error_with_code(
                            SEMA_TYPE_MISMATCH,
                            "Data-enum case labels do not support ranges",
                            "Match enum variants directly, for example `Shape.Circle(R)` or `Shape.Point`.",
                            *span,
                        );
                        return Some(Vec::new());
                    }
                    return Some(self.check_data_enum_pattern(case_ty, start));
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

                self.check_destructure_variant(case_ty, *variant, *span);
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

    /// Validate a data-enum pattern and extract any bindings it introduces.
    ///
    /// **Documentation:** `docs/pascal/06-pattern-matching.md`
    pub(super) fn check_data_enum_pattern(
        &mut self,
        case_ty: &Ty,
        expr: &Expr,
    ) -> Vec<(String, Ty)> {
        self.collect_enum_pattern_bindings(case_ty, expr, false)
    }

    fn collect_enum_pattern_bindings(
        &mut self,
        expected_ty: &Ty,
        expr: &Expr,
        allow_binding: bool,
    ) -> Vec<(String, Ty)> {
        match expr {
            Expr::Call {
                designator, args, ..
            } => self.collect_variant_pattern_bindings(expected_ty, designator, args),
            Expr::Designator(designator) if allow_binding && designator.parts.len() == 1 => {
                match &designator.parts[0] {
                    DesignatorPart::Ident(name, _) if name == "_" => Vec::new(),
                    DesignatorPart::Ident(name, _) => vec![(name.clone(), expected_ty.clone())],
                    DesignatorPart::Index(_, _) => Vec::new(),
                }
            }
            Expr::Designator(designator) => {
                self.collect_variant_pattern_bindings(expected_ty, designator, &[])
            }
            Expr::Integer(..) | Expr::Real(..) | Expr::Str(..) | Expr::Bool(..)
                if allow_binding =>
            {
                let actual_ty = self.check_expr(expr);
                self.check_type_compat(
                    expected_ty,
                    &actual_ty,
                    "enum pattern literal",
                    expr_span(expr),
                );
                Vec::new()
            }
            _ if allow_binding => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    "Enum pattern fields only allow bindings, `_`, literals, or nested enum patterns",
                    "Use a binding name, `_`, a literal, or another enum pattern such as `Inner.A(X)`.",
                    expr_span(expr),
                );
                Vec::new()
            }
            _ => {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    "Data-enum case labels must be enum variant patterns",
                    "Use `Variant(...)`, `Type.Variant(...)`, or a fieldless variant like `Type.Point`.",
                    expr_span(expr),
                );
                Vec::new()
            }
        }
    }

    fn collect_variant_pattern_bindings(
        &mut self,
        expected_ty: &Ty,
        designator: &Designator,
        args: &[Expr],
    ) -> Vec<(String, Ty)> {
        let Some(variant_name) = variant_name_from_designator(designator) else {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Enum pattern must name a variant",
                "Use a variant pattern such as `Shape.Circle(R)` or `Inner.B`.",
                designator.span,
            );
            return Vec::new();
        };

        let (variant_name_owned, field_types) = {
            let Some(enum_ty) = self.resolve_enum_ty(expected_ty) else {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Nested enum pattern expects an enum field, found `{expected_ty:?}`"),
                    "Use `_`, a binding name, or a literal when the field is not an enum.",
                    designator.span,
                );
                return Vec::new();
            };

            let Some(variant) = enum_ty
                .variants
                .iter()
                .find(|variant| variant.name.eq_ignore_ascii_case(variant_name))
            else {
                let valid_variants = enum_ty
                    .variants
                    .iter()
                    .map(|variant| variant.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!(
                        "Pattern variant `{variant_name}` does not belong to enum `{}`",
                        enum_ty.name
                    ),
                    format!(
                        "Use one of the variants of `{}`: {valid_variants}.",
                        enum_ty.name
                    ),
                    designator.span,
                );
                return Vec::new();
            };

            (
                variant.name.clone(),
                variant
                    .fields
                    .iter()
                    .map(|(_, field_ty)| field_ty.clone())
                    .collect::<Vec<_>>(),
            )
        };

        if args.len() != field_types.len() {
            self.error_with_code(
                SEMA_ENUM_FIELD_COUNT_MISMATCH,
                format!(
                    "Variant '{}' expects {} field{}, but {} {} supplied.",
                    variant_name_owned,
                    field_types.len(),
                    if field_types.len() == 1 { "" } else { "s" },
                    args.len(),
                    if args.len() == 1 { "was" } else { "were" },
                ),
                format!(
                    "Use {} binding{} to match all fields of '{}'.",
                    field_types.len(),
                    if field_types.len() == 1 { "" } else { "s" },
                    variant_name_owned,
                ),
                designator.span,
            );
            return Vec::new();
        }

        let mut bindings = Vec::new();
        for (arg, field_ty) in args.iter().zip(field_types.iter()) {
            bindings.extend(self.collect_enum_pattern_bindings(field_ty, arg, true));
        }
        bindings
    }

    fn check_destructure_variant(&mut self, case_ty: &Ty, variant: DestructureVariant, span: Span) {
        let valid = matches!(
            (case_ty, variant),
            (
                Ty::Result(_, _),
                DestructureVariant::Ok | DestructureVariant::Error
            ) | (
                Ty::Option(_),
                DestructureVariant::Some | DestructureVariant::None
            ) | (Ty::Error, _)
        );
        if valid {
            return;
        }

        let hint = match variant {
            DestructureVariant::Ok | DestructureVariant::Error => {
                "Use `Ok(Value)` and `Error(Value)` with `Result`."
            }
            DestructureVariant::Some | DestructureVariant::None => {
                "Use `Some(Value)` and `None` with `Option`."
            }
        };
        self.error_with_code(
            SEMA_TYPE_MISMATCH,
            format!("Pattern `{variant:?}` does not match case expression type `{case_ty:?}`"),
            hint,
            span,
        );
    }

    fn resolve_enum_ty<'a>(&'a self, ty: &'a Ty) -> Option<&'a EnumTy> {
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

fn variant_name_from_designator(designator: &Designator) -> Option<&str> {
    designator.parts.last().and_then(|part| match part {
        DesignatorPart::Ident(name, _) => Some(name.as_str()),
        DesignatorPart::Index(_, _) => None,
    })
}

pub(super) fn binding_type_for_variant(case_ty: &Ty, variant: &DestructureVariant) -> Ty {
    match (case_ty, variant) {
        (Ty::Result(ok, _), DestructureVariant::Ok) => *ok.clone(),
        (Ty::Result(_, err), DestructureVariant::Error) => *err.clone(),
        (Ty::Option(inner), DestructureVariant::Some) => *inner.clone(),
        _ => Ty::Error,
    }
}
