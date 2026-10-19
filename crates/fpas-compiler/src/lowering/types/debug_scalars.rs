//! Declared scalar display names that runtime type erasure would otherwise discard.

use fpas_parser::TypeExpr;
use fpas_sema::Ty;

use super::TypeTable;

pub(super) fn collect_names(
    metadata: &fpas_sema::AnalysisMetadata,
) -> std::collections::BTreeMap<String, String> {
    metadata
        .named_types
        .iter()
        .filter_map(|(name, ty)| {
            declared_name(ty).map(|display| (name.to_ascii_lowercase(), display))
        })
        .collect()
}

pub(super) fn declared_name(ty: &Ty) -> Option<String> {
    match ty {
        Ty::Distinct(distinct) => Some(distinct.name.clone()),
        Ty::Subrange(range) => Some(range.name.clone()),
        Ty::Enum(enumeration) => Some(enumeration.name.clone()),
        _ => None,
    }
}

impl TypeTable {
    pub(in crate::lowering) fn declared_semantic_scalar_name(&self, ty: &Ty) -> Option<String> {
        declared_name(ty).or_else(|| match ty {
            Ty::Named(name) => self.lookup_scalar_name(name),
            _ => None,
        })
    }

    /// Resolve nominal display metadata without changing the stored runtime type.
    pub(in crate::lowering) fn declared_scalar_name(&self, ty: &TypeExpr) -> Option<String> {
        let TypeExpr::Named { id, .. } = ty else {
            return None;
        };
        let name = id.parts.join(".");
        self.lookup_scalar_name(&name)
    }

    fn lookup_scalar_name(&self, name: &str) -> Option<String> {
        if let Some(display) = self.debug_scalar_names.get(&name.to_ascii_lowercase()) {
            return Some(display.clone());
        }
        let mut candidates = self
            .debug_scalar_names
            .iter()
            .filter(|(candidate, _)| super::super::type_names::matches(candidate, name))
            .map(|(_, display)| display);
        let result = candidates.next()?;
        candidates.next().is_none().then(|| result.clone())
    }
}
