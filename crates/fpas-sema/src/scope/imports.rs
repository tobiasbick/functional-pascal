//! Visibility and canonical identities of source-local import namespaces.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`

use std::collections::{BTreeMap, HashMap, HashSet};

/// Unit identities and aliases used in one source file.
#[derive(Debug, Default)]
pub(crate) struct ImportNames {
    /// Canonical unit name to source alias spelling.
    pub(crate) aliases: BTreeMap<String, String>,
    units_by_alias: HashMap<String, String>,
    plain_units: HashSet<String>,
    known_units: HashSet<String>,
}

impl ImportNames {
    pub(crate) fn add_plain(&mut self, unit: &str) {
        self.note_unit(unit);
        self.plain_units.insert(unit.to_ascii_lowercase());
    }

    pub(crate) fn add_alias(&mut self, unit: &str, alias: &str) {
        self.note_unit(unit);
        let unit = unit.to_ascii_lowercase();
        self.units_by_alias
            .insert(alias.to_ascii_lowercase(), unit.clone());
        self.aliases.insert(unit, alias.to_string());
    }

    pub(crate) fn is_alias(&self, name: &str) -> bool {
        self.units_by_alias.contains_key(&name.to_ascii_lowercase())
    }

    pub(crate) fn note_unit(&mut self, unit: &str) {
        self.known_units.insert(unit.to_ascii_lowercase());
    }

    /// Resolves alias prefixes while keeping linked names available to internal metadata.
    pub(crate) fn canonical_name(&self, name: &str) -> String {
        let (root, suffix) = name.split_once('.').unwrap_or((name, ""));
        if let Some(unit) = self.units_by_alias.get(&root.to_ascii_lowercase()) {
            return if suffix.is_empty() {
                unit.clone()
            } else {
                format!("{unit}.{}", suffix.to_ascii_lowercase())
            };
        }
        name.to_ascii_lowercase()
    }

    /// Resolves a source name, rejecting canonical paths hidden by an alias.
    pub(crate) fn visible_name(&self, name: &str) -> Option<String> {
        let canonical = name.to_ascii_lowercase();
        if self.aliases.is_empty() {
            return Some(canonical);
        }
        let (root, _) = canonical.split_once('.').unwrap_or((&canonical, ""));
        if let Some(unit) = self.units_by_alias.get(root) {
            let resolved = self.canonical_name(name);
            // An alias denotes one unit, not other units beneath its namespace.
            if self
                .known_units
                .iter()
                .any(|other| other.len() > unit.len() && matches_unit(&resolved, other))
            {
                return None;
            }
            return Some(resolved);
        }
        self.hidden_unit(&canonical).is_none().then_some(canonical)
    }

    pub(crate) fn hidden_path_hint(&self, name: &str) -> Option<String> {
        let canonical = name.to_ascii_lowercase();
        let hidden = self.hidden_unit(&canonical)?;
        let alias = &self.aliases[hidden];
        let suffix = &name[hidden.len()..];
        Some(format!(
            "This unit is imported as `{alias}`. Use `{alias}{suffix}`; the original unit path is not visible for an aliased import."
        ))
    }

    fn hidden_unit(&self, name: &str) -> Option<&String> {
        let matches = |unit: &&String| matches_unit(name, unit);
        let hidden = self
            .aliases
            .keys()
            .filter(matches)
            .max_by_key(|unit| unit.len())?;
        if self
            .plain_units
            .iter()
            .filter(matches)
            .any(|unit| unit.len() > hidden.len())
        {
            return None;
        }
        Some(hidden)
    }
}

fn matches_unit(name: &str, unit: &str) -> bool {
    name.strip_prefix(unit)
        .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('.'))
}
