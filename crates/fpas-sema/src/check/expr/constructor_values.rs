//! Payload variant references use ordinary readonly callable signatures.
//! Documentation: `docs/pascal/language/functions/function-types.md`.

use crate::{
    check::Checker,
    scope::SymbolKind,
    types::{FunctionTy, ParamTy, Ty},
};
use fpas_parser::Designator;

impl Checker {
    pub(super) fn constructor_callable_type(&self, designator: &Designator, ty: Ty) -> Ty {
        let name = Self::resolve_designator_name(designator);
        if !self
            .scopes
            .lookup(&name)
            .is_some_and(|symbol| symbol.kind == SymbolKind::EnumVariantConstructor)
        {
            return ty;
        }
        let Ty::Enum(enumeration) = self.resolve_visible_type(&ty) else {
            return ty;
        };
        let variant_name = name.rsplit('.').next().unwrap_or(&name);
        let Some(variant) = enumeration
            .variants
            .iter()
            .find(|variant| variant.name.eq_ignore_ascii_case(variant_name))
        else {
            return Ty::Error;
        };
        Ty::Function(FunctionTy {
            type_params: if enumeration.type_args.is_empty() {
                enumeration.type_params.clone()
            } else {
                vec![]
            },
            params: variant
                .fields
                .iter()
                .map(|(name, ty)| ParamTy::value(name, ty.clone()))
                .collect(),
            return_type: Box::new(Ty::Enum(enumeration)),
            variadic: false,
        })
    }
}
