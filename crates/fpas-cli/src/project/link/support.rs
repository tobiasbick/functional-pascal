use super::UnitFile;
use crate::project::common::qualified_id_to_string;

use fpas_parser::QualifiedId;
use std::collections::HashMap;

pub(super) fn collect_std_uses(uses: &[QualifiedId]) -> Vec<QualifiedId> {
    let mut std_uses = Vec::<QualifiedId>::new();
    merge_std_uses(&mut std_uses, uses);
    std_uses
}

pub(super) fn merge_std_uses(target: &mut Vec<QualifiedId>, from: &[QualifiedId]) {
    for used in from {
        if !is_std_unit(used) {
            continue;
        }
        let key = canonical_unit_key(used);
        if target
            .iter()
            .any(|existing| canonical_unit_key(existing) == key)
        {
            continue;
        }
        target.push(used.clone());
    }
}

pub(super) fn is_std_unit(used: &QualifiedId) -> bool {
    used.parts
        .first()
        .is_some_and(|head| head.eq_ignore_ascii_case("std"))
}

pub(super) fn canonical_unit_key(id: &QualifiedId) -> String {
    qualified_id_to_string(id).to_ascii_lowercase()
}

pub(super) fn display_unit_key(key: &str) -> String {
    let mut result = String::new();
    for (i, segment) in key.split('.').enumerate() {
        if i > 0 {
            result.push('.');
        }
        let mut chars = segment.chars();
        if let Some(first) = chars.next() {
            result.push(first.to_ascii_uppercase());
            result.push_str(chars.as_str());
        }
    }
    result
}

pub(super) fn internal_link_error(unit_key: &str, context: &str) -> String {
    format!(
        "Internal linker error: unit `{}` disappeared while {context}.\n  help: This indicates inconsistent project graph construction.",
        display_unit_key(unit_key)
    )
}

pub(super) fn internal_symbol_error(unit_key: &str) -> String {
    format!(
        "Internal linker error: symbols for unit `{}` were not collected before import rewriting.\n  help: This indicates inconsistent linker state.",
        display_unit_key(unit_key)
    )
}

pub(super) fn unknown_unit_error(
    key: &str,
    units: &HashMap<String, UnitFile>,
    owner: &str,
) -> String {
    let mut known = units
        .values()
        .map(|unit| qualified_id_to_string(&unit.unit.name))
        .collect::<Vec<_>>();
    known.sort();
    let display = display_unit_key(key);
    if known.is_empty() {
        format!(
            "Unknown unit `{display}` in {owner}. No source units are available in the project."
        )
    } else {
        format!(
            "Unknown unit `{display}` in {owner}.\n  help: Available units: {}.",
            known.join(", ")
        )
    }
}
