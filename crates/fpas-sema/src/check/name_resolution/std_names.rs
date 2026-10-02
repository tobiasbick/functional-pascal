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

        None
    }

    /// Hint for an ambiguous short routine name in a call with `arg_count` arguments.
    ///
    /// With at least one argument, the method form on that argument selects the routine by the
    /// argument's type, for example `Value.Unwrap()` for an option or a result.
    pub(crate) fn ambiguous_call_hint(&self, name: &str, arg_count: usize) -> Option<String> {
        let hint = self.ambiguous_hint(name)?;
        let _ = arg_count;
        Some(hint)
    }

    /// Resolves standard call names while preserving lexical shadowing of imported aliases.
    ///
    /// **Documentation:** `docs/pascal/program-structure/units.md`.
    pub(crate) fn builtin_std_dispatch_name(&self, name: &str) -> String {
        self.scopes
            .lookup_original_name(name)
            .unwrap_or(name)
            .to_owned()
    }

    /// Registers the qualified symbols of a `Std.*` unit named in `uses` on first qualified use.
    ///
    /// A qualified name of a unit missing from `uses` stays unresolved, so the lookup reports it
    /// with a hint to import the unit; qualification never imports a unit implicitly.
    ///
    /// **Documentation:** `docs/pascal/program-structure/units.md`.
    pub(crate) fn ensure_fq_std_unit_loaded(&mut self, fully_qualified_name: &str) {
        let Some((unit, _)) = crate::std_units::parse_std_qualified_call(fully_qualified_name)
        else {
            return;
        };

        if !self.loaded_std_units.contains(&unit)
            || self.scopes.lookup(fully_qualified_name).is_some()
        {
            return;
        }

        crate::std_registry::register_single_std_unit(self, unit.as_str());
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
