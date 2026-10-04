//! Contextual typing for constructors, collections, and value decisions.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use super::Checker;
use crate::types::Ty;
use fpas_parser::Expr;

impl Checker {
    /// Propagate declared context through selected values, wrappers, and collections.
    pub(crate) fn check_expr_with_expected(&mut self, expr: &Expr, expected: &Ty) -> Ty {
        if let Expr::InvalidRecord(span) = expr {
            let ty = self.reject_obsolete_record(*span, Some(expected));
            return self.annotate_expected_expression(expr, ty);
        }
        let decision = match expr {
            Expr::If(decision) => Some(self.check_if_expression(decision, Some(expected))),
            Expr::Case(decision) => Some(self.check_case_expression(decision, Some(expected))),
            _ => None,
        };
        if let Some(ty) = decision {
            return self.annotate_expected_expression(expr, ty);
        }
        if let Some(ty) = self.try_check_enum_construction(expr, Some(expected)) {
            return ty;
        }
        if let Some(ty) = self.try_check_record_construction(expr, Some(expected)) {
            return ty;
        }
        let resolved = self.resolve_visible_type(expected);
        let contextual = match (expr, &resolved) {
            (Expr::Paren(inner, _), _) => Some(self.check_expr_with_expected(inner, expected)),
            (Expr::OptionNone(_), Ty::Option(_)) => Some(resolved.clone()),
            (Expr::OptionSome(inner, _), Ty::Option(expected)) => {
                let actual = self.check_expr_with_expected(inner, expected);
                self.check_type_compat(expected, &actual, "Option payload", inner.span());
                Some(Ty::Option(Box::new(
                    expected.complete_inference_with(&actual),
                )))
            }
            (Expr::ResultOk(inner, _), Ty::Result(expected, error)) => {
                let actual = self.check_expr_with_expected(inner, expected);
                self.check_type_compat(expected, &actual, "Result payload", inner.span());
                Some(Ty::Result(
                    Box::new(expected.complete_inference_with(&actual)),
                    error.clone(),
                ))
            }
            (Expr::ResultError(inner, _), Ty::Result(ok, expected)) => {
                let actual = self.check_expr_with_expected(inner, expected);
                self.check_type_compat(expected, &actual, "Result error payload", inner.span());
                Some(Ty::Result(
                    ok.clone(),
                    Box::new(expected.complete_inference_with(&actual)),
                ))
            }
            (Expr::ArrayLiteral(elements, _), Ty::Array(expected)) => {
                let mut element_type = (**expected).clone();
                for element in elements {
                    let actual = self.check_expr_with_expected(element, expected);
                    self.check_type_compat(&element_type, &actual, "array element", element.span());
                    element_type = element_type.complete_inference_with(&actual);
                }
                Some(Ty::Array(Box::new(element_type)))
            }
            (Expr::DictLiteral(entries, _), Ty::Dict(key, value)) => {
                let mut key_type = (**key).clone();
                let mut value_type = (**value).clone();
                for (actual_key, actual_value) in entries {
                    let actual = self.check_expr_with_expected(actual_key, key);
                    self.check_type_compat(&key_type, &actual, "dictionary key", actual_key.span());
                    key_type = key_type.complete_inference_with(&actual);
                    let actual = self.check_expr_with_expected(actual_value, value);
                    self.check_type_compat(
                        &value_type,
                        &actual,
                        "dictionary value",
                        actual_value.span(),
                    );
                    value_type = value_type.complete_inference_with(&actual);
                }
                self.check_dictionary_key_type(&key_type, expr.span());
                Some(Ty::Dict(Box::new(key_type), Box::new(value_type)))
            }
            _ => None,
        };
        if let Some(ty) = contextual {
            return self.annotate_expected_expression(expr, ty);
        }
        let actual = self.check_expr(expr);
        if let Some(instantiated) =
            self.instantiate_callable_with_expected(&actual, &resolved, expr.span())
        {
            self.annotate_expected_expression(expr, instantiated)
        } else {
            actual
        }
    }

    fn annotate_expected_expression(&mut self, expr: &Expr, ty: Ty) -> Ty {
        let key = Self::expr_lookup_key(expr);
        self.expr_types.insert(key, ty.clone());
        self.propagate_task_bound_expr(expr, key);
        ty
    }
}
