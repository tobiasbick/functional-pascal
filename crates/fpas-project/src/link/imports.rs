use super::{
    UnitFile,
    rewrite::declaration_name,
    rewrite::linked_decl_name,
    support::{canonical_unit_key, internal_link_error, is_std_unit, unknown_unit_error},
};
use crate::common::qualified_id_to_string;

use fpas_parser::{QualifiedId, Visibility};
use std::collections::{BTreeSet, HashMap, HashSet};

pub(super) fn collect_unit_exports(
    reachable: &HashSet<String>,
    units: &HashMap<String, UnitFile>,
) -> Result<HashMap<String, HashMap<String, String>>, String> {
    let mut exports = HashMap::<String, HashMap<String, String>>::new();

    for unit_key in reachable {
        let Some(unit_file) = units.get(unit_key) else {
            return Err(internal_link_error(unit_key, "collecting unit exports"));
        };
        let unit_name = qualified_id_to_string(&unit_file.unit.name);

        let mut unit_exports = HashMap::<String, String>::new();
        for decl in &unit_file.unit.declarations {
            if decl.visibility() == Visibility::Private {
                continue;
            }
            let short_name = declaration_name(decl);
            if unit_exports.contains_key(short_name) {
                return Err(format!(
                    "Duplicate declaration `{short_name}` in unit `{unit_name}`.\n  help: Use unique top-level declaration names per unit."
                ));
            }
            unit_exports.insert(
                short_name.to_string(),
                linked_decl_name(&unit_name, short_name, decl.visibility()),
            );
        }

        exports.insert(unit_key.clone(), unit_exports);
    }

    Ok(exports)
}

pub(super) fn collect_all_unit_symbols(
    reachable: &HashSet<String>,
    units: &HashMap<String, UnitFile>,
) -> Result<HashMap<String, HashMap<String, String>>, String> {
    let mut all = HashMap::<String, HashMap<String, String>>::new();

    for unit_key in reachable {
        let Some(unit_file) = units.get(unit_key) else {
            return Err(internal_link_error(unit_key, "collecting unit symbols"));
        };
        let unit_name = qualified_id_to_string(&unit_file.unit.name);

        let mut symbols = HashMap::<String, String>::new();
        for decl in &unit_file.unit.declarations {
            let short_name = declaration_name(decl);
            if symbols.contains_key(short_name) {
                return Err(format!(
                    "Duplicate declaration `{short_name}` in unit `{unit_name}`.\n  help: Use unique top-level declaration names per unit."
                ));
            }
            symbols.insert(
                short_name.to_string(),
                linked_decl_name(&unit_name, short_name, decl.visibility()),
            );
        }

        all.insert(unit_key.clone(), symbols);
    }

    Ok(all)
}

pub(super) struct ImportMap {
    pub(super) resolved: HashMap<String, String>,
    pub(super) ambiguous: HashMap<String, Vec<String>>,
}

pub(super) fn build_imports(
    uses: &[QualifiedId],
    include_self: Option<&HashMap<String, String>>,
    exports: &HashMap<String, HashMap<String, String>>,
    units: &HashMap<String, UnitFile>,
) -> Result<ImportMap, String> {
    let mut candidates = HashMap::<String, BTreeSet<String>>::new();

    for used in uses {
        if is_std_unit(used) {
            continue;
        }
        let key = canonical_unit_key(used);
        let Some(unit_exports) = exports.get(&key) else {
            return Err(unknown_unit_error(
                &key,
                units,
                "uses clause during import resolution",
            ));
        };
        add_export_candidates(&mut candidates, unit_exports);
    }

    if let Some(self_exports) = include_self {
        add_export_candidates(&mut candidates, self_exports);
    }

    let mut resolved = HashMap::<String, String>::new();
    let mut ambiguous = HashMap::<String, Vec<String>>::new();
    for (short, values) in candidates {
        if values.len() == 1 {
            if let Some(only) = values.iter().next() {
                resolved.insert(short, only.clone());
            }
        } else {
            ambiguous.insert(short, values.into_iter().collect());
        }
    }

    Ok(ImportMap {
        resolved,
        ambiguous,
    })
}

fn add_export_candidates(
    candidates: &mut HashMap<String, BTreeSet<String>>,
    exports: &HashMap<String, String>,
) {
    for (short, qualified) in exports {
        candidates
            .entry(short.clone())
            .or_default()
            .insert(qualified.clone());
    }
}
