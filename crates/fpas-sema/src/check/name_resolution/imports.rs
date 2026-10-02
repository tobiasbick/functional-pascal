//! Explicit import aliases and source-level qualification.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`.

use super::Checker;
use fpas_diagnostics::codes::{SEMA_DUPLICATE_DECLARATION, SEMA_UNKNOWN_NAME};
use fpas_lexer::Span;
use fpas_parser::Import;
use std::collections::HashSet;

impl Checker {
    /// Installs one reserved source alias for each directly imported unit.
    pub(crate) fn register_import_aliases(&mut self, imports: &[Import]) {
        let mut units = HashSet::new();
        for import in imports {
            let unit = import.parts.join(".");
            let alias = import.alias.to_ascii_lowercase();
            if !units.insert(unit.to_ascii_lowercase()) {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!("Unit `{unit}` is imported more than once"),
                    "Keep one `uses Unit.Name as Alias;` declaration per unit.",
                    import.span,
                );
            }
            if self.import_aliases.contains_key(&alias) {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!("Duplicate import alias `{}`", import.alias),
                    "Choose a distinct alias; names are case-insensitive.",
                    import.alias_span,
                );
                continue;
            }
            if self.scopes.lookup(&alias).is_some() {
                self.error_with_code(
                    SEMA_DUPLICATE_DECLARATION,
                    format!(
                        "Import alias `{}` conflicts with a predefined name",
                        import.alias
                    ),
                    "Choose an alias distinct from primitive type names such as `integer`.",
                    import.alias_span,
                );
            }
            self.scopes.reserve_import_alias(&alias, import.alias_span);
            self.import_aliases.insert(alias, unit);
        }
    }

    /// Reports declarations that attempted to shadow a reserved import alias.
    pub(crate) fn report_import_alias_conflicts(&mut self) {
        for (name, span) in std::mem::take(&mut self.scopes.import_alias_conflicts) {
            self.error_with_code(SEMA_DUPLICATE_DECLARATION,
                format!("Declaration `{name}` conflicts with an import alias"),
                "Rename the declaration or the import alias; aliases cannot be shadowed in any lexical scope.", span);
        }
    }

    /// Translates a source alias to its canonical linked unit name.
    pub(crate) fn qualified_import_name(&self, name: &str) -> String {
        let Some((root, member)) = name.split_once('.') else {
            return name.to_owned();
        };
        self.import_aliases
            .get(&root.to_ascii_lowercase())
            .map_or_else(|| name.to_owned(), |unit| format!("{unit}.{member}"))
    }

    /// Resolves source qualification while enforcing direct import access paths.
    pub(crate) fn resolve_source_name(&mut self, name: &str, span: Span) -> String {
        let resolved = self.qualified_import_name(name);
        if name
            .split_once('.')
            .is_some_and(|(root, _)| self.import_aliases.contains_key(&root.to_ascii_lowercase()))
        {
            if let Some((unit, member)) = name.split_once('.').and_then(|(alias, member)| {
                self.import_aliases
                    .get(&alias.to_ascii_lowercase())
                    .map(|unit| (unit.clone(), member))
            }) {
                let root = member.split('.').next().unwrap_or(member);
                let namespace = format!("{unit}.{root}");
                if self.scopes.lookup(&namespace).is_none() {
                    let unit_prefix = format!("{}.", unit.to_ascii_lowercase());
                    let canonical = resolved.to_ascii_lowercase();
                    let nested = self
                        .used_unit_names
                        .iter()
                        .chain(&self.supporting_unit_names)
                        .filter(|known| {
                            known.starts_with(&unit_prefix)
                                && canonical
                                    .strip_prefix(known.as_str())
                                    .is_some_and(|rest| rest.starts_with('.'))
                        })
                        .max_by_key(|known| known.len());
                    if let Some(nested) = nested {
                        let namespace = resolved[..nested.len()].to_owned();
                        if !self.errors.iter().any(|error| {
                            error.span == Some(span.diagnostic_span_or_synthetic())
                                && error.message.starts_with("Import alias in ")
                        }) {
                            self.error_with_code(SEMA_UNKNOWN_NAME,
                                format!("Import alias in `{name}` names `{unit}`, not `{namespace}`"),
                                format!("Import `{namespace}` with its own alias and access its members through that alias."),
                                span);
                        }
                    }
                }
            }
            return resolved;
        }
        // Interface constants have synthetic spans and already use linked names.
        if span.length == 0 {
            return resolved;
        }
        let replacement = self.import_aliases.iter().find_map(|(alias, unit)| {
            name.to_ascii_lowercase()
                .strip_prefix(&format!("{}.", unit.to_ascii_lowercase()))
                .map(|member| format!("{alias}.{member}"))
        });
        if let Some(replacement) = replacement {
            if !self.errors.iter().any(|error| {
                error.span == Some(span.diagnostic_span_or_synthetic())
                    && error.message.contains("import alias")
            }) {
                self.error_with_code(
                    SEMA_UNKNOWN_NAME,
                    format!("Use the import alias to access `{name}`"),
                    format!("Write `{replacement}`; an import exposes only its declared alias."),
                    span,
                );
            }
        } else if self
            .supporting_unit_names
            .iter()
            .any(|unit| name.to_ascii_lowercase().starts_with(&format!("{unit}.")))
        {
            self.error_with_code(
                SEMA_UNKNOWN_NAME,
                format!("Unit containing `{name}` is not directly imported"),
                "Declare `uses Unit.Name as Alias;` and access the type through `Alias.TypeName`.",
                span,
            );
        }
        resolved
    }

    /// Suggests alias-qualified alternatives without opening imported short names.
    pub(crate) fn import_name_hint(&self, name: &str) -> Option<String> {
        let mut candidates = Vec::new();
        for (alias, unit) in &self.import_aliases {
            if unit
                .rsplit('.')
                .next()
                .is_some_and(|short| short.eq_ignore_ascii_case(name))
            {
                candidates.push(format!("{alias}.Member"));
            }
            for (qualified, _) in self.scopes.root_symbols_with_prefix(&format!("{unit}.")) {
                let member = &qualified[unit.len() + 1..];
                if member.eq_ignore_ascii_case(name)
                    || member
                        .rsplit('.')
                        .next()
                        .is_some_and(|short| short.eq_ignore_ascii_case(name))
                {
                    candidates.push(format!("{alias}.{member}"));
                }
            }
        }
        candidates.sort();
        candidates.dedup();
        (!candidates.is_empty()).then(|| {
            format!(
                "Imports open no short names. Use {}.",
                candidates
                    .iter()
                    .map(|name| format!("`{name}`"))
                    .collect::<Vec<_>>()
                    .join(" or ")
            )
        })
    }
}
