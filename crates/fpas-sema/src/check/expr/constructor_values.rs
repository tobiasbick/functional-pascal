//! Payload variant references use ordinary readonly callable signatures.
//! Documentation: `docs/pascal/language/functions/function-types.md`.

use crate::{check::Checker, scope::SymbolKind, types::Ty};
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
        enumeration
            .constructor_type(variant_name)
            .map(Ty::Function)
            .unwrap_or(Ty::Error)
    }
}
