//! Constant comparisons and their coverage representation in patterns.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/syntax.md`

use super::{Checker, Pat, PatternBindings};
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_parser::{DesignatorPart, Expr};
use fpas_unit::interface::ConstantValue;

impl Checker {
    /// Checks a fieldless variant, a literal, or a named constant compared with the value.
    pub(in crate::check::stmt::control_flow::if_case) fn check_value_pattern(
        &mut self,
        expected_ty: &Ty,
        expr: &Expr,
        bindings: &mut PatternBindings,
    ) -> Pat {
        let bare_name = match expr {
            Expr::Designator(designator) => match designator.parts.as_slice() {
                [DesignatorPart::Ident(name, _)] => Some(name.as_str()),
                _ => None,
            },
            _ => None,
        };
        if bare_name == Some("_") {
            self.reject_whole_value_wildcard(expr.span());
            return Pat::Wild;
        }
        if let Some(enum_ty) = self.resolve_enum_ty(expected_ty)
            && enum_ty.has_data()
        {
            let Expr::Designator(designator) = expr else {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Expected a variant of enum `{}`", enum_ty.name),
                    "Match a variant, for example `Shape.Circle(const R)` or `Shape.Point`.",
                    expr.span(),
                );
                return Pat::Wild;
            };
            if let Some(name) = bare_name
                && self.scopes.lookup(name).is_none()
                && !enum_ty
                    .variants
                    .iter()
                    .any(|variant| variant.name.eq_ignore_ascii_case(name))
            {
                return self.implicit_binding(name, expected_ty, expr.span(), bindings);
            }
            return self.check_variant_pattern(expected_ty, designator, &[], expr.span(), bindings);
        }
        if let Some(name) = bare_name
            && self.scopes.lookup(name).is_none()
        {
            return self.implicit_binding(name, expected_ty, expr.span(), bindings);
        }

        let value_ty = self.check_expr_with_expected(expr, Some(expected_ty));
        let errors_before = self.errors.len();
        self.check_type_compat(expected_ty, &value_ty, "pattern value", expr.span());
        if value_ty.is_error() || self.errors.len() != errors_before {
            return Pat::Wild;
        }
        self.require_case_constant(expr);
        let compared_ty = super::super::distinct_labels::comparison_type(expected_ty);
        let comparable = self.resolve_enum_ty(expected_ty).is_some()
            || compared_ty.is_ordinal()
            || compared_ty.compatible_with(&Ty::String)
            || compared_ty.is_error();
        if !comparable {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("A pattern cannot compare values of type `{expected_ty}`"),
                "Compare ordinal, enum, or string values in a pattern; bind other values with `const Name` and test them in a guard.",
                expr.span(),
            );
            return Pat::Wild;
        }
        self.value_constructor(expected_ty, expr)
    }

    /// Normalizes compared constants so finite coverage and duplicate checks share their values.
    pub(in crate::check::stmt::control_flow::if_case) fn value_constructor(
        &self,
        expected_ty: &Ty,
        expr: &Expr,
    ) -> Pat {
        match self.scalar_constant_value(expr) {
            Some(ConstantValue::Boolean(value)) => Pat::Ctor(value.to_string(), Vec::new()),
            Some(ConstantValue::EnumValue { variant_name, .. })
                if self.resolve_enum_ty(expected_ty).is_some() =>
            {
                Pat::Ctor(variant_name.to_ascii_lowercase(), Vec::new())
            }
            Some(value @ (ConstantValue::Integer(_) | ConstantValue::String(_))) => {
                Pat::Value(value)
            }
            _ => Pat::Other,
        }
    }
}
