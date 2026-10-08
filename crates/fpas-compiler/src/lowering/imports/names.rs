//! Source-local bindings for canonical linked unit exports.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`

use std::collections::BTreeMap;

/// Resolve intrinsic constants through the same source alias as interface constants.
pub(super) fn install_constants(
    aliases: &BTreeMap<String, String>,
    constants: &mut BTreeMap<String, fpas_ir::Constant>,
) {
    for (unit, alias) in aliases {
        let Some(canonical) = fpas_std::STD_UNITS_INTRINSIC
            .iter()
            .find(|name| name.eq_ignore_ascii_case(unit))
        else {
            continue;
        };
        for symbol in fpas_std::std_unit_symbols(canonical) {
            let value = match super::super::builtin_constants::value(symbol) {
                Some(fpas_bytecode::Value::Integer(value)) => fpas_ir::Constant::Integer(value),
                Some(fpas_bytecode::Value::Real(value)) => fpas_ir::Constant::Real(value),
                _ => continue,
            };
            if let Some((_, name)) = symbol.rsplit_once('.') {
                constants.insert(format!("{alias}.{name}").to_ascii_lowercase(), value);
            }
        }
    }
}

/// Adds source alias keys without allocating new callable or global identities.
pub(super) fn install<T: Clone>(
    aliases: &BTreeMap<String, String>,
    bindings: &mut BTreeMap<String, T>,
) {
    let mut additions = Vec::new();
    for (unit, alias) in aliases {
        let prefix = format!("{unit}.");
        for (name, binding) in bindings.iter() {
            if let Some(tail) = name.strip_prefix(&prefix) {
                additions.push((
                    format!("{}.{}", alias.to_ascii_lowercase(), tail),
                    binding.clone(),
                ));
            }
        }
    }
    bindings.extend(additions);
}
