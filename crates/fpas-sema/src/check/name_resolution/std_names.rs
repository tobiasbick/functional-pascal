use super::Checker;
use crate::scope::canonical_symbol_name;
use fpas_diagnostics::codes::SEMA_UNQUALIFIED_NAME;

impl Checker {
    /// Lists required source qualifiers without making the short spellings legal.
    ///
    /// **Documentation:** `docs/pascal/tools/diagnostics.md` (FP3043).
    pub(crate) fn qualification_hint(&self, name: &str) -> Option<String> {
        let leaf = name.rsplit('.').next().unwrap_or(name);
        let alias_prefix = name.split_once('.').and_then(|(root, _)| {
            self.scopes
                .imports
                .is_alias(root)
                .then(|| format!("{root}."))
        });
        let candidates = self
            .imported_candidates
            .get(&canonical_symbol_name(name))
            .or_else(|| {
                (!name.contains('.') || alias_prefix.is_some())
                    .then(|| self.imported_candidates.get(&canonical_symbol_name(leaf)))
                    .flatten()
            })?;
        let mut spellings = candidates
            .iter()
            .cloned()
            .filter(|candidate| !candidate.eq_ignore_ascii_case(name))
            .filter(|candidate| {
                alias_prefix.as_ref().is_none_or(|prefix| {
                    candidate
                        .to_ascii_lowercase()
                        .starts_with(&prefix.to_ascii_lowercase())
                })
            })
            .collect::<Vec<_>>();
        spellings.sort_by_key(|candidate| candidate.to_ascii_lowercase());
        spellings.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
        (!spellings.is_empty()).then(|| format!("Use the required alias/type qualifier: {}. Imported names require `Alias.Name`; user enum variants require `Type.Variant` or `Alias.Type.Variant`.", spellings.iter().map(|candidate| format!("`{candidate}`")).collect::<Vec<_>>().join(", ")))
    }

    /// Resolves standard call names while preserving lexical shadowing of imported aliases.
    ///
    /// **Documentation:** `docs/pascal/program-structure/units.md`.
    pub(crate) fn builtin_std_dispatch_name(&self, name: &str) -> String {
        if !name.contains('.') {
            return name.to_string();
        }
        self.scopes
            .lookup_original_name(name)
            .unwrap_or(name)
            .to_string()
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
            || self
                .scopes
                .imports
                .lookup_metadata(fully_qualified_name)
                .is_some()
        {
            return;
        }

        crate::std_registry::register_single_std_unit(self, unit.as_str());
    }

    /// Report an imported type whose source-local alias qualifier is missing.
    pub(crate) fn report_unqualified_type_name(&mut self, name: &str, span: fpas_lexer::Span) {
        if let Some(hint) = self.qualification_hint(name) {
            self.error_with_code(
                SEMA_UNQUALIFIED_NAME,
                format!("Type `{name}` requires its import alias"),
                hint,
                span,
            );
        }
    }
}
