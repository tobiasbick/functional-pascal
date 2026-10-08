//! Canonical nominal definitions for recursive references and member access.
//!
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`

use super::super::super::Checker;
use crate::types::Ty;
use std::collections::HashSet;

impl Checker {
    /// Resolve a stored type identity without letting value shadowing hide its definition.
    pub(crate) fn resolve_visible_type(&self, ty: &Ty) -> Ty {
        let mut resolved = ty.clone();
        let mut visited = HashSet::new();
        while let Ty::Named(name) = &resolved {
            if !visited.insert(name.to_ascii_lowercase()) {
                break;
            }
            let Some(symbol) = self.scopes.lookup_type(name) else {
                break;
            };
            resolved = symbol.ty.clone();
        }
        let name = match &resolved {
            Ty::Record(record) => &record.name,
            Ty::Enum(enumeration) => &enumeration.name,
            _ => return resolved,
        };
        self.scopes
            .lookup_type(name)
            .map(|symbol| symbol.ty.clone())
            .unwrap_or(resolved)
    }
}
