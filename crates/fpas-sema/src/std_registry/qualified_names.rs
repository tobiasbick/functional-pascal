//! Diagnostic candidates for mandatory alias-qualified standard-library names.
//! **Documentation:** `docs/pascal/program-structure/units.md`.

use crate::check::Checker;
use crate::scope::canonical_symbol_name;

/// Retain qualified repair targets without opening any short source bindings.
pub(crate) fn register_qualified_import_candidates(checker: &mut Checker) {
    let units: Vec<_> = checker.loaded_std_units.iter().cloned().collect();
    for unit in units {
        let prefix = format!("{unit}.");
        for (name, symbol) in checker.scopes.imports.metadata_symbols_with_prefix(&prefix) {
            let Some(spelling) = checker.scopes.imports.define_source(&unit, &name, symbol) else {
                continue;
            };
            let suffix = &name[prefix.len()..];
            for key in [suffix, suffix.rsplit('.').next().unwrap_or(suffix)] {
                let candidates = checker
                    .imported_candidates
                    .entry(canonical_symbol_name(key))
                    .or_default();
                if !candidates
                    .iter()
                    .any(|known| known.eq_ignore_ascii_case(&spelling))
                {
                    candidates.push(spelling.clone());
                }
            }
        }
    }
}
