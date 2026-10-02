use super::Checker;
use crate::std_units::hint_for_unknown_std_name;
use fpas_parser::{Designator, DesignatorPart};

mod imports;
mod std_names;
mod types;

impl Checker {
    pub(crate) fn hint_unknown_callable(&self, name: &str) -> String {
        if let Some(hint) = self.import_name_hint(name) {
            return hint;
        }
        if let Some((unit, _)) = name.rsplit_once('.')
            && !unit.to_ascii_lowercase().starts_with("std.")
            && self.used_unit_names.contains(&unit.to_ascii_lowercase())
        {
            return "Check that the symbol is public. Private unit members are not visible outside their unit.".to_owned();
        }
        hint_for_unknown_std_name(name, &self.loaded_std_units)
    }

    pub(crate) fn resolve_designator_name(&mut self, designator: &Designator) -> String {
        self.resolve_source_name(&Self::designator_name(designator), designator.span)
    }

    pub(crate) fn designator_name(designator: &Designator) -> String {
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
