//! Lexical and qualified enum-constructor resolution for patterns.
//!
//! **Documentation:** `docs/pascal/language/pattern-matching/enum-patterns.md`

use super::Checker;
use crate::scope::SymbolKind;
use crate::types::Ty;
use fpas_diagnostics::codes::SEMA_TYPE_MISMATCH;
use fpas_parser::{Designator, DesignatorPart};

impl Checker {
    /// Resolves the full constructor name and requires a variant of the matched enum.
    pub(super) fn variant_field_types(
        &mut self,
        expected_ty: &Ty,
        constructor: &Designator,
    ) -> Option<(String, Vec<Ty>)> {
        let Some(DesignatorPart::Ident(variant_name, _)) = constructor.parts.last() else {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                "Enum pattern must name a variant",
                "Use a variant pattern such as `Shape.Circle(const R)` or `Inner.B`.",
                constructor.span,
            );
            return None;
        };
        let Some(enum_ty) = self.resolve_enum_ty(expected_ty) else {
            if !expected_ty.is_error() {
                self.error_with_code(
                    SEMA_TYPE_MISMATCH,
                    format!("Variant pattern `{variant_name}` does not match a value of type `{expected_ty}`"),
                    "Use a variant pattern only where the matched value is an enum.",
                    constructor.span,
                );
            }
            return None;
        };

        let mut resolved_ty = self.check_designator_expr(constructor);
        if resolved_ty.is_error() {
            return None;
        }
        let name = Self::resolve_designator_name(constructor);
        let is_variant = self.scopes.lookup(&name).is_some_and(|symbol| {
            matches!(
                symbol.kind,
                SymbolKind::EnumMember | SymbolKind::EnumVariantConstructor
            )
        });
        if !is_variant {
            self.error_with_code(
                SEMA_TYPE_MISMATCH,
                format!("Pattern constructor `{name}` must name an enum variant"),
                format!(
                    "Use a qualified variant of `{}`, such as `{}.{}(...)`.",
                    enum_ty.name, enum_ty.name, variant_name
                ),
                constructor.span,
            );
            return None;
        }
        if let Ty::Enum(declaration) = &resolved_ty
            && declaration.name.eq_ignore_ascii_case(&enum_ty.name)
            && !declaration.type_params.is_empty()
            && declaration.type_args.is_empty()
        {
            resolved_ty = Ty::Enum(std::sync::Arc::new(
                declaration.instantiate(enum_ty.type_args.clone()),
            ));
        }
        let errors_before = self.errors.len();
        self.check_type_compat(
            expected_ty,
            &resolved_ty,
            "pattern variant",
            constructor.span,
        );
        if self.errors.len() != errors_before {
            return None;
        }
        let variant = enum_ty
            .variants
            .iter()
            .find(|variant| variant.name.eq_ignore_ascii_case(variant_name))?;
        Some((
            variant.name.clone(),
            variant.fields.iter().map(|(_, ty)| ty.clone()).collect(),
        ))
    }
}
