//! Source namespaces and canonical unit identities for editor lookup.
//!
//! See `docs/pascal/program-structure/units.md`.

use fpas_parser::Import;

use super::NavigationDocument;

impl NavigationDocument {
    /// Finds a direct import by case-insensitive canonical unit identity.
    pub(crate) fn import_for(&self, owner: &str) -> Option<&Import> {
        self.uses
            .iter()
            .find(|import| import.unit.parts.join(".").eq_ignore_ascii_case(owner))
    }

    /// Returns whether the unit is available, including through an alias.
    pub(crate) fn uses_owner(&self, owner: &str) -> bool {
        self.owner.eq_ignore_ascii_case(owner) || self.import_for(owner).is_some()
    }

    /// Returns the source namespace used for the unit, preserving alias spelling.
    pub(crate) fn namespace_for(&self, owner: &str) -> Option<String> {
        if let Some(import) = self.import_for(owner) {
            return import.alias.as_ref().map(|alias| alias.name.clone());
        }
        if self.owner.eq_ignore_ascii_case(owner)
            && matches!(
                self.snapshot.compilation_unit(),
                fpas_parser::CompilationUnit::Unit(_)
            )
        {
            return Some(self.owner.clone());
        }
        None
    }

    /// Expands a source-local alias in a qualified type name.
    pub(crate) fn canonical_name(&self, name: &str) -> String {
        let (first, tail) = name.split_once('.').unwrap_or((name, ""));
        if let Some(import) = self.uses.iter().find(|import| {
            import
                .alias
                .as_ref()
                .is_some_and(|alias| alias.name.eq_ignore_ascii_case(first))
        }) {
            let owner = import.unit.parts.join(".");
            return if tail.is_empty() {
                owner
            } else {
                format!("{owner}.{tail}")
            };
        }
        name.to_owned()
    }

    /// Tests only the contextual modifier between a unit path and its alias.
    pub(crate) fn is_import_modifier(&self, offset: usize) -> bool {
        self.uses.iter().any(|import| {
            import.alias.as_ref().is_some_and(|alias| {
                import.unit.span.offset + import.unit.span.length <= offset
                    && offset < alias.span.offset
            })
        })
    }
}
