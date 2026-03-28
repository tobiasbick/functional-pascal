use super::Checker;
use fpas_diagnostics::codes::SEMA_AMBIGUOUS_IMPORTED_NAME;

impl Checker {
    /// If `name` is an ambiguous short import, return a hint listing the candidates.
    pub(crate) fn ambiguous_hint(&self, name: &str) -> Option<String> {
        self.ambiguous_imports.get(name).map(|candidates| {
            format!(
                "`{name}` exists in multiple imported units: {}. Use the fully qualified name to disambiguate.",
                candidates.join(", ")
            )
        })
    }

    /// Loads a `Std.*` unit on demand when code uses a fully qualified name without `uses`.
    pub(crate) fn builtin_std_dispatch_name(&self, name: &str) -> String {
        if name.contains('.') {
            name.to_string()
        } else if let Some(qualified) = self.short_builtin_redirect.get(name) {
            qualified.clone()
        } else {
            name.to_string()
        }
    }

    pub(crate) fn ensure_fq_std_unit_loaded(&mut self, fully_qualified_name: &str) {
        let Some((unit, _)) = crate::std_units::parse_std_qualified_call(fully_qualified_name)
        else {
            return;
        };

        let already_loaded = self.loaded_std_units.contains(&unit);
        let symbol_reachable = already_loaded && self.scopes.lookup(fully_qualified_name).is_some();
        if symbol_reachable {
            return;
        }

        self.loaded_std_units.insert(unit.clone());
        crate::std_registry::register_single_std_unit(self, unit.as_str());
        crate::std_registry::register_short_aliases(self);
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
