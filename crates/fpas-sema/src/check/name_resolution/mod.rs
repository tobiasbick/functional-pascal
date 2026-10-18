use super::Checker;
use crate::std_units::hint_for_unknown_std_name;
use fpas_parser::{Designator, DesignatorPart, Expr};

mod parser_migration;
mod std_names;
mod types;

pub(super) use types::PendingKey;

impl Checker {
    /// Gives a qualified repair for unresolved calls without exposing hidden unit symbols.
    pub(crate) fn hint_unknown_callable(&self, name: &str, args: &[Expr]) -> String {
        if let Some(hint) = self.foreign_builtin_hint(name, args) {
            return hint;
        }
        if let Some(hint) = self.record_constructor_hint(name, args) {
            return hint;
        }
        if let Some(hint) = self.parser_migration_hint(name) {
            return hint;
        }
        if let Some(hint) = self.scopes.imports.hidden_path_hint(name) {
            return hint;
        }
        if let Some((root, _)) = name.split_once('.')
            && self.scopes.imports.is_alias(root)
        {
            return format!(
                "Check the public symbol name after import alias `{root}`. Private unit members are not visible outside their unit."
            );
        }
        if let Some((unit, _)) = name.rsplit_once('.')
            && !unit.to_ascii_lowercase().starts_with("std.")
            && self.used_unit_names.contains(&unit.to_ascii_lowercase())
        {
            return "Check that the symbol is public. Private unit members are not visible outside their unit.".to_owned();
        }
        hint_for_unknown_std_name(name, &self.loaded_std_units)
    }

    /// Returns the fix for an unresolved call over `span` that is a foreign built-in or a
    /// record `T.Create(…)`, matching the help of [`Self::hint_unknown_callable`].
    pub(crate) fn unknown_callable_fix(
        &self,
        name: &str,
        args: &[Expr],
        span: fpas_lexer::Span,
    ) -> Option<fpas_diagnostics::Fix> {
        self.foreign_builtin_fix(name, args, span)
            .or_else(|| self.record_constructor_fix(name, args, span))
    }

    pub(crate) fn resolve_designator_name(designator: &Designator) -> String {
        let mut result = String::new();
        for part in &designator.parts {
            if let DesignatorPart::Ident(name, _) = part {
                if !result.is_empty() {
                    result.push('.');
                }
                result.push_str(name);
            }
        }
        result
    }
}
