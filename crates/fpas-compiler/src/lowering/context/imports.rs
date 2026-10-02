//! Source aliases translated to linked unit identities.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`.

use super::LoweringContext;

impl LoweringContext {
    /// Translate a source alias into the canonical linked unit prefix.
    pub(in crate::lowering) fn qualified_import_name(&self, name: &str) -> String {
        let Some((root, member)) = name.split_once('.') else {
            return name.to_owned();
        };
        self.import_aliases
            .get(&root.to_ascii_lowercase())
            .map_or_else(|| name.to_owned(), |unit| format!("{unit}.{member}"))
    }
}
