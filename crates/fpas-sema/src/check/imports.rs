//! Direct-import uniqueness and source namespace diagnostics.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`

use std::collections::HashSet;

use fpas_diagnostics::codes::SEMA_DUPLICATE_DECLARATION;
use fpas_lexer::Span;
use fpas_parser::Import;

use super::Checker;

impl Checker {
    /// Validate mandatory aliases before installing any direct-import namespace.
    pub(super) fn initialize_imports(&mut self, imports: &[Import], owner_root: Option<&str>) {
        self.scopes.imports = Default::default();
        let mut roots = imports
            .iter()
            .filter_map(|used| used.unit.parts.first())
            .map(|root| root.to_ascii_lowercase())
            .collect::<HashSet<_>>();
        roots.insert("std".to_string());
        if let Some(root) = owner_root {
            roots.insert(root.to_ascii_lowercase());
        }
        let mut seen = HashSet::new();
        let mut aliases = HashSet::new();
        for used in imports {
            let unit = used.unit.parts.join(".");
            if !seen.insert(unit.to_ascii_lowercase()) {
                self.error_with_code(SEMA_DUPLICATE_DECLARATION,
                    format!("Repeated import of unit `{unit}`"),
                    "Import each unit once per source file using `uses Unit.Path as Alias;`; remove the repeated entry.", used.span);
                continue;
            }
            let Some(alias) = &used.alias else {
                self.error_with_code(
                    fpas_diagnostics::codes::SEMA_UNKNOWN_NAME,
                    format!("Import `{unit}` requires an explicit alias"),
                    "Write `uses Unit.Path as Alias;` and access public names through `Alias.Name`.",
                    used.span,
                );
                continue;
            };
            let key = alias.name.to_ascii_lowercase();
            if !aliases.insert(key.clone())
                || roots.contains(&key)
                || self.scopes.lookup(&alias.name).is_some()
            {
                self.error_with_code(SEMA_DUPLICATE_DECLARATION,
                    format!("Import alias `{}` conflicts with another alias or name", alias.name),
                    "Choose an alias different from other aliases, declared names, and unit namespace roots; names are case-insensitive.", alias.span);
                continue;
            }
            self.scopes.imports.add_alias(&unit, &alias.name);
        }
    }

    /// Keep source-local aliases reserved throughout every lexical scope.
    pub(crate) fn check_import_alias_collision(&mut self, name: &str, span: Span) -> bool {
        if !self.scopes.imports.is_alias(name) {
            return false;
        }
        self.error_with_code(SEMA_DUPLICATE_DECLARATION,
            format!("Declared name `{name}` conflicts with an import alias"),
            "Rename the declaration or the import alias. An import alias cannot be shadowed by a local name or parameter.", span);
        true
    }
}
