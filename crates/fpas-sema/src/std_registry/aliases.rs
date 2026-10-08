//! Short names for imported `Std.*` and source-unit symbols.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md` and `docs/pascal/std/README.md` (from the repository root).

use crate::check::Checker;
use crate::scope::canonical_symbol_name;
use crate::scope::{Symbol, SymbolKind};
use std::collections::HashMap;

/// Register unqualified (short) aliases for all imported `Std.*` and source-unit symbols.
///
/// For each loaded unit `Std.X`, every symbol `Std.X.Sym` is a short-name candidate `Sym`;
/// directly imported source units contribute the candidates collected in
/// `Checker::source_short_candidates`. A short name with exactly one candidate becomes an alias;
/// with several it is recorded as ambiguous (error only at point of use, not at the `uses`
/// site), whether the candidates come from `Std.*`, from source units, or from both.
///
/// A type hides enum variants of the same short name, as within one unit: the variant stays
/// reachable through its qualified `Type.Variant` name. Other names stay ambiguous with a variant.
///
/// Re-running this after more units become loaded rebuilds the map: previous short bindings
/// inserted here are removed from the program root scope first so a name that becomes ambiguous
/// is no longer bound to a stale symbol.
pub fn register_short_aliases(checker: &mut Checker) {
    for key in std::mem::take(&mut checker.std_short_alias_keys) {
        checker.scopes.remove_from_root(&key);
    }
    for key in std::mem::take(&mut checker.source_short_alias_keys) {
        checker.scopes.remove_from_root(&key);
    }
    checker.ambiguous_imports.clear();
    checker.short_builtin_redirect.clear();

    // canonical short name -> (display short name, [(qualified_name, symbol), ...])
    let mut short_map: HashMap<String, (String, Vec<(String, Symbol)>)> = HashMap::new();

    let units: Vec<String> = checker.loaded_std_units.iter().cloned().collect();
    for unit in &units {
        if checker
            .scopes
            .imports
            .aliases
            .contains_key(&unit.to_ascii_lowercase())
        {
            continue;
        }
        let prefix = format!("{unit}.");
        let qualified_names = checker.scopes.names_with_prefix(&prefix);
        for qname in qualified_names {
            let short = &qname[prefix.len()..];
            if short.is_empty() {
                continue;
            }
            if let Some(sym) = checker.scopes.lookup(&qname) {
                let sym = sym.clone();
                let candidates = checker
                    .imported_candidates
                    .entry(canonical_symbol_name(short))
                    .or_default();
                if !candidates
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(&qname))
                {
                    candidates.push(qname.clone());
                }
                short_map
                    .entry(canonical_symbol_name(short))
                    .or_insert_with(|| (short.to_string(), Vec::new()))
                    .1
                    .push((qname, sym));
            }
        }
    }

    for (key, candidates) in &checker.source_short_candidates {
        short_map
            .entry(key.clone())
            .or_insert_with(|| (key.clone(), Vec::new()))
            .1
            .extend(candidates.iter().cloned());
    }

    for (short_key, (short, entries)) in short_map {
        let entries = visible_candidates(entries);
        if let [(qualified, sym)] = entries.as_slice() {
            let from_std = is_std_name(qualified);
            if from_std && sym.kind == SymbolKind::BuiltinStd {
                checker
                    .short_builtin_redirect
                    .insert(short_key.clone(), qualified.clone());
            }
            let discard = checker.scopes.discard_info(qualified);
            if checker.scopes.define_in_root(&short, sym.clone()) {
                checker.scopes.set_discard_info(&short, discard);
                if from_std {
                    checker.std_short_alias_keys.insert(short_key);
                } else {
                    checker.source_short_alias_keys.insert(short_key);
                }
            }
        } else {
            let mut qualified_names: Vec<String> = entries
                .into_iter()
                .map(|(qualified, _)| qualified)
                .collect();
            qualified_names.sort_by(|left, right| {
                canonical_symbol_name(left)
                    .cmp(&canonical_symbol_name(right))
                    .then_with(|| left.cmp(right))
            });
            checker.ambiguous_imports.insert(short_key, qualified_names);
        }
    }
}

/// Removes duplicate candidates and enum variants hidden by a type of the same short name.
fn visible_candidates(mut entries: Vec<(String, Symbol)>) -> Vec<(String, Symbol)> {
    entries.sort_by(|left, right| {
        canonical_symbol_name(&left.0)
            .cmp(&canonical_symbol_name(&right.0))
            .then_with(|| left.0.cmp(&right.0))
    });
    entries.dedup_by(|left, right| left.0.eq_ignore_ascii_case(&right.0));
    let is_variant = |symbol: &Symbol| {
        matches!(
            symbol.kind,
            SymbolKind::EnumMember | SymbolKind::EnumVariantConstructor
        )
    };
    if entries
        .iter()
        .any(|(_, symbol)| symbol.kind == SymbolKind::Type)
    {
        entries.retain(|(_, symbol)| !is_variant(symbol));
    }
    entries
}

fn is_std_name(qualified: &str) -> bool {
    qualified
        .split('.')
        .next()
        .is_some_and(|root| root.eq_ignore_ascii_case("std"))
}
