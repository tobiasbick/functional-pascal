use super::Checker;
use crate::scope::canonical_symbol_name;
use fpas_diagnostics::codes::SEMA_AMBIGUOUS_IMPORTED_NAME;

impl Checker {
    /// If `name` is an ambiguous short import or enum variant, return a hint listing the candidates.
    pub(crate) fn ambiguous_hint(&self, name: &str) -> Option<String> {
        let canonical_name = canonical_symbol_name(name);
        if let Some(candidates) = self.ambiguous_enum_variants.get(&canonical_name) {
            return Some(format!(
                "`{name}` exists in multiple enums: {}. Use the fully qualified variant name to disambiguate.",
                candidates.join(", ")
            ));
        }

        self.ambiguous_imports.get(&canonical_name).map(|candidates| {
            format!(
                "`{name}` exists in multiple imported units: {}. Use the fully qualified name to disambiguate.",
                candidates.join(", ")
            )
        })
    }

    /// Loads a `Std.*` unit on demand when code uses a fully qualified name without `uses`.
    pub(crate) fn builtin_std_dispatch_name(&self, name: &str) -> String {
        let canonical = canonical_symbol_name(name);
        if let Some(qualified) = self.short_builtin_redirect.get(&canonical) {
            return qualified.clone();
        }
        if name.contains('.') {
            return self
                .scopes
                .lookup_original_name(name)
                .unwrap_or(name)
                .to_string();
        }
        let mut candidates = self
            .loaded_std_units
            .iter()
            .flat_map(|unit| fpas_std::std_unit_symbols(unit))
            .filter(|qualified| {
                qualified
                    .rsplit_once('.')
                    .is_some_and(|(_, short)| short.eq_ignore_ascii_case(name))
            })
            .copied()
            .collect::<Vec<_>>();
        candidates.sort_unstable();
        candidates.dedup();
        if let [qualified] = candidates.as_slice() {
            return (*qualified).to_string();
        }
        name.to_string()
    }

    pub(crate) fn ensure_fq_std_unit_loaded(&mut self, fully_qualified_name: &str) {
        let Some((unit, _)) = crate::std_units::parse_std_qualified_call(fully_qualified_name)
        else {
            return;
        };

        if self.loaded_std_units.contains(&unit)
            && self.scopes.lookup(fully_qualified_name).is_some()
        {
            return;
        }

        let new_unit = !self.loaded_std_units.contains(&unit);
        self.loaded_std_units.insert(unit.clone());
        crate::std_registry::register_single_std_unit(self, unit.as_str());
        if new_unit {
            crate::std_registry::register_short_aliases(self);
        }
    }

    pub(crate) fn report_ambiguous_type_name(&mut self, name: &str, span: fpas_lexer::Span) {
        if let Some(hint) = self.ambiguous_hint(name) {
            self.error_with_code(
                SEMA_AMBIGUOUS_IMPORTED_NAME,
                format!("Ambiguous type `{name}`"),
                hint,
                span,
            );
        }
    }
}
