//! Public source aliases and canonical imported metadata have separate namespaces.
//! See `docs/pascal/program-structure/units.md`.

use std::collections::{BTreeMap, HashMap};

use super::{Symbol, SymbolKind};

#[derive(Debug)]
struct ImportedSymbol {
    original_name: String,
    symbol: Symbol,
}

/// Imports belonging to one source file; supporting metadata never exposes source names.
#[derive(Debug, Default)]
pub(crate) struct ImportNames {
    /// Canonical direct unit name to its written source alias.
    pub(crate) aliases: BTreeMap<String, String>,
    units_by_alias: HashMap<String, String>,
    source_symbols: HashMap<String, ImportedSymbol>,
    metadata_symbols: HashMap<String, ImportedSymbol>,
}

impl ImportNames {
    /// Install one explicit source-local alias.
    pub(crate) fn add_alias(&mut self, unit: &str, alias: &str) {
        let unit = unit.to_ascii_lowercase();
        self.units_by_alias
            .insert(alias.to_ascii_lowercase(), unit.clone());
        self.aliases.insert(unit, alias.to_owned());
    }

    /// Whether this identifier denotes a directly imported namespace.
    pub(crate) fn is_alias(&self, name: &str) -> bool {
        self.units_by_alias.contains_key(&name.to_ascii_lowercase())
    }

    /// Canonicalize a source alias for linked identities and initialization summaries.
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

    /// Store imported metadata without creating a source binding.
    pub(crate) fn define_metadata(&mut self, name: &str, symbol: Symbol) -> bool {
        let key = name.to_ascii_lowercase();
        if self
            .metadata_symbols
            .get(&key)
            .is_some_and(|entry| entry.symbol.kind == SymbolKind::Type)
        {
            return false;
        }
        self.metadata_symbols.insert(
            key,
            ImportedSymbol {
                original_name: name.to_owned(),
                symbol,
            },
        );
        true
    }

    /// Install one direct public symbol through its exact owning unit's alias.
    pub(crate) fn define_source(
        &mut self,
        unit: &str,
        name: &str,
        symbol: Symbol,
    ) -> Option<String> {
        let spelling = self.source_name_for(unit, name)?;
        self.source_symbols.insert(
            spelling.to_ascii_lowercase(),
            ImportedSymbol {
                original_name: name.to_owned(),
                symbol,
            },
        );
        Some(spelling)
    }

    /// Render an exported name using its owning unit, even when a child unit shares its prefix.
    pub(crate) fn source_name_for(&self, unit: &str, name: &str) -> Option<String> {
        let alias = self.aliases.get(&unit.to_ascii_lowercase())?;
        let suffix = name.get(unit.len()..)?;
        (name.get(..unit.len())?.eq_ignore_ascii_case(unit) && suffix.starts_with('.'))
            .then(|| format!("{alias}{suffix}"))
    }

    /// Resolve only a directly imported public source spelling.
    pub(crate) fn lookup_source(&self, name: &str) -> Option<&Symbol> {
        self.source_symbols
            .get(&name.to_ascii_lowercase())
            .map(|entry| &entry.symbol)
    }

    /// Return the linked name retained by a source binding.
    pub(crate) fn original_source_name(&self, name: &str) -> Option<&str> {
        self.source_symbols
            .get(&name.to_ascii_lowercase())
            .map(|entry| entry.original_name.as_str())
    }

    /// Resolve a stored canonical identity independently of source aliases and local values.
    pub(crate) fn lookup_metadata(&self, name: &str) -> Option<&Symbol> {
        self.metadata_symbols
            .get(&name.to_ascii_lowercase())
            .map(|entry| &entry.symbol)
    }

    /// Retain canonical layouts and their visible alias spellings for compiler lowering.
    pub(crate) fn type_symbols(&self) -> BTreeMap<String, super::Ty> {
        self.metadata_symbols
            .iter()
            .chain(self.source_symbols.iter())
            .filter(|(_, entry)| entry.symbol.kind == SymbolKind::Type)
            .map(|(name, entry)| (name.clone(), entry.symbol.ty.clone()))
            .collect()
    }

    /// List canonical intrinsic declarations for registration and the editor catalog.
    pub(crate) fn metadata_symbols_with_prefix(&self, prefix: &str) -> Vec<(String, Symbol)> {
        let prefix = prefix.to_ascii_lowercase();
        let mut symbols = self
            .metadata_symbols
            .iter()
            .filter(|(name, _)| name.starts_with(&prefix))
            .map(|(_, entry)| (entry.original_name.clone(), entry.symbol.clone()))
            .collect::<Vec<_>>();
        symbols.sort_by(|left, right| left.0.cmp(&right.0));
        symbols
    }

    /// Explain the explicit alias required for a removed original unit path.
    pub(crate) fn hidden_path_hint(&self, name: &str) -> Option<String> {
        let canonical = name.to_ascii_lowercase();
        let (unit, alias) = self
            .aliases
            .iter()
            .filter(|(unit, _)| matches_unit(&canonical, unit))
            .max_by_key(|(unit, _)| unit.len())?;
        Some(format!(
            "This unit is imported as `{alias}`. Use `{alias}{suffix}`; the original unit path is not visible for an aliased import.",
            suffix = &name[unit.len()..]
        ))
    }
}

fn matches_unit(name: &str, unit: &str) -> bool {
    name.strip_prefix(unit)
        .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with('.'))
}
