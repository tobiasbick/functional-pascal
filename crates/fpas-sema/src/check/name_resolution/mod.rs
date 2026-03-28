use super::Checker;
use crate::std_units::hint_for_unknown_std_name;
use fpas_parser::{Designator, DesignatorPart};

mod std_names;
mod types;

impl Checker {
    pub(crate) fn hint_unknown_callable(&self, name: &str) -> String {
        hint_for_unknown_std_name(name, &self.loaded_std_units)
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
