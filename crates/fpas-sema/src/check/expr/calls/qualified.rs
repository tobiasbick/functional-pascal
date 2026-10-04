//! Qualified unit calls remain distinct from record field calls.

use crate::check::Checker;
use fpas_parser::{Designator, DesignatorPart};

impl Checker {
    /// Distinguishes a qualified imported-unit call from a value receiver call.
    pub(in crate::check) fn designator_has_unit_prefix(&self, designator: &Designator) -> bool {
        let Some((_, prefix)) = designator.parts.split_last() else {
            return false;
        };
        let names = prefix
            .iter()
            .map(|part| match part {
                DesignatorPart::Ident(name, _) => Some(name.as_str()),
                DesignatorPart::Index(_, _) => None,
            })
            .collect::<Option<Vec<_>>>();
        names.is_some_and(|names| {
            (self
                .used_unit_names
                .contains(&names.join(".").to_ascii_lowercase())
                || (names.len() == 1
                    && names.first().is_some_and(|root| {
                        self.import_aliases.contains_key(&root.to_ascii_lowercase())
                    })))
                && names
                    .first()
                    .is_some_and(|root| self.scopes.lookup(root).is_none())
        })
    }
}
