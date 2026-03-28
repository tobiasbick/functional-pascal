use crate::check::Checker;
use crate::scope::{Symbol, SymbolKind};
use std::collections::HashMap;

/// Register unqualified (short) aliases for all imported `Std.*` symbols.
///
/// For each loaded unit `Std.X`, every symbol `Std.X.Sym` gets a short alias
/// `Sym` - unless another loaded unit also exports a symbol with the same
/// short name, in which case the name is recorded as ambiguous (error only
/// at point of use, not at the `uses` site).
pub fn register_short_aliases(checker: &mut Checker) {
    let units: Vec<String> = checker.loaded_std_units.iter().cloned().collect();

    // short_name -> [(qualified_name, symbol), ...]
    let mut short_map: HashMap<String, Vec<(String, Symbol)>> = HashMap::new();

    for unit in &units {
        let prefix = format!("{unit}.");
        let qualified_names = checker.scopes.names_with_prefix(&prefix);
        for qname in qualified_names {
            let short = &qname[prefix.len()..];
            if short.is_empty() {
                continue;
            }
            if let Some(sym) = checker.scopes.lookup(&qname) {
                let sym = sym.clone();
                short_map
                    .entry(short.to_string())
                    .or_default()
                    .push((qname, sym));
            }
        }
    }

    for (short, entries) in short_map {
        if entries.len() == 1 {
            let (qualified, sym) = &entries[0];
            if sym.kind == SymbolKind::BuiltinStd {
                checker
                    .short_builtin_redirect
                    .insert(short.clone(), qualified.clone());
            }
            checker.scopes.define(&short, sym.clone());
        } else {
            let qualified_names: Vec<String> = entries.into_iter().map(|(q, _)| q).collect();
            checker.ambiguous_imports.insert(short, qualified_names);
        }
    }
}
