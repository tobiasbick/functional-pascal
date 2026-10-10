//! Contextual construction of enum variants using ordinary qualified names.
//! See `docs/pascal/language/types/enums.md` and `docs/pascal/language/types/generics.md`.

use super::construction_inference::ConstructorField;
use crate::{
    check::{Checker, calls::CallTarget},
    types::{ParamTy, Ty},
};
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_lexer::Span;
use fpas_parser::Expr;
use std::sync::Arc;

impl Checker {
    /// Map variant fields, infer the enum application, and check concrete payloads.
    pub(super) fn check_enum_construction(
        &mut self,
        name: &str,
        enum_ty: &Ty,
        args: &[Expr],
        span: Span,
        expected: Option<&Ty>,
    ) -> Ty {
        let Ty::Enum(enumeration) = self.resolve_visible_type(enum_ty) else {
            self.check_args_only(args);
            return Ty::Error;
        };
        let variant_name = name.rsplit('.').next().unwrap_or(name);
        let Some(variant) = enumeration
            .variants
            .iter()
            .find(|variant| variant.name.eq_ignore_ascii_case(variant_name))
        else {
            self.check_args_only(args);
            return Ty::Error;
        };
        let fields: Vec<_> = variant
            .fields
            .iter()
            .map(|(name, ty)| ParamTy::value(name, ty.clone()))
            .collect();
        let Some(ordered) = self.order_call_arguments(
            name,
            CallTarget::EnumVariant,
            &fields,
            false,
            &args.iter().collect::<Vec<_>>(),
        ) else {
            return Ty::Error;
        };
        if ordered.len() != fields.len() {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!(
                    "`{name}` expects {} argument(s), got {}",
                    fields.len(),
                    ordered.len()
                ),
                format!(
                    "Provide values for: {}",
                    fields
                        .iter()
                        .map(|field| field.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                span,
            );
        }
        let context = expected.map(|ty| self.resolve_visible_type(ty));
        let context = match &context {
            Some(Ty::Enum(context)) if context.name.eq_ignore_ascii_case(&enumeration.name) => {
                Some(context)
            }
            _ => None,
        };
        let instance = if enumeration.type_params.is_empty() {
            enumeration.clone()
        } else {
            let contextual_variant = context.and_then(|context| {
                context
                    .variants
                    .iter()
                    .find(|variant| variant.name.eq_ignore_ascii_case(variant_name))
            });
            let supplied: Vec<_> = ordered
                .iter()
                .zip(&variant.fields)
                .enumerate()
                .map(|(index, (value, (_, declared)))| ConstructorField {
                    value,
                    declared,
                    expected: contextual_variant
                        .and_then(|variant| variant.fields.get(index))
                        .map(|(_, ty)| ty),
                })
                .collect();
            let fixed = if enumeration.type_args.is_empty() {
                context.map_or(&enumeration.type_args, |context| &context.type_args)
            } else {
                &enumeration.type_args
            };
            let arguments = self.infer_constructor_arguments(
                &enumeration.name,
                &enumeration.type_params,
                fixed,
                &supplied,
                span,
            );
            let template = self
                .scopes
                .lookup_type(&enumeration.name)
                .and_then(|symbol| match &symbol.ty {
                    Ty::Enum(template) => Some(template.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| enumeration.clone());
            Arc::new(template.instantiate(arguments))
        };
        if let Some(variant) = instance
            .variants
            .iter()
            .find(|variant| variant.name.eq_ignore_ascii_case(variant_name))
        {
            for (value, (_, field_ty)) in ordered.iter().zip(&variant.fields) {
                let actual = self.check_expr_with_expected(value, Some(field_ty));
                self.check_type_compat(
                    field_ty,
                    &actual,
                    &format!("`{name}` argument"),
                    value.span(),
                );
            }
        }
        for value in ordered.iter().skip(fields.len()) {
            self.check_expr(value);
        }
        for value in ordered {
            self.prechecked_receivers
                .remove(&Self::expr_lookup_key(value));
        }
        Ty::Enum(instance)
    }
}
