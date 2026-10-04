use super::Checker;

impl Checker {
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
}
