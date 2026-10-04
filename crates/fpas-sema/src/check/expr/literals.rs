//! Array and dictionary component inference.
//!
//! **Documentation:** `docs/pascal/language/types/arrays.md`,
//! `docs/pascal/language/types/dictionaries.md` and `docs/pascal/language/types/records.md`.

use super::Checker;
use crate::types::Ty;
use fpas_parser::Expr;

impl Checker {
    /// Infer every array element, completing empty constructors from later values.
    pub(super) fn check_array_literal(&mut self, elements: &[Expr]) -> Ty {
        let Some(first) = elements.first() else {
            return Ty::Array(Box::new(Ty::Error));
        };

        let mut element_ty = self.check_expr(first);
        for element in &elements[1..] {
            let actual_ty = self.check_expr(element);
            self.check_type_compat(&element_ty, &actual_ty, "array element", element.span());
            element_ty = element_ty.complete_inference_with(&actual_ty);
        }

        Ty::Array(Box::new(element_ty))
    }

    /// Infer dictionary keys and values from all entries.
    pub(super) fn check_dict_literal(&mut self, pairs: &[(Expr, Expr)]) -> Ty {
        let Some((first_key, first_value)) = pairs.first() else {
            return Ty::Dict(Box::new(Ty::Error), Box::new(Ty::Error));
        };

        let mut key_ty = self.check_expr(first_key);
        let mut value_ty = self.check_expr(first_value);
        for (key, value) in &pairs[1..] {
            let actual_key = self.check_expr(key);
            self.check_type_compat(&key_ty, &actual_key, "dict key", key.span());
            key_ty = key_ty.complete_inference_with(&actual_key);
            let actual_value = self.check_expr(value);
            self.check_type_compat(&value_ty, &actual_value, "dict value", value.span());
            value_ty = value_ty.complete_inference_with(&actual_value);
        }

        self.check_dictionary_key_type(&key_ty, first_key.span());
        Ty::Dict(Box::new(key_ty), Box::new(value_ty))
    }
}
