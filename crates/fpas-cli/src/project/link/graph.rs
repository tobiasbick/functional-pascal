use super::{
    UnitFile,
    support::{
        canonical_unit_key, display_unit_key, internal_link_error, is_std_unit, unknown_unit_error,
    },
};
use crate::project::common::qualified_id_to_string;

use fpas_parser::QualifiedId;
use std::collections::{HashMap, HashSet};

pub(super) fn resolve_reachable_units(
    root_uses: &[QualifiedId],
    units: &HashMap<String, UnitFile>,
) -> Result<HashSet<String>, String> {
    let mut queue = Vec::<String>::new();
    let mut reachable = HashSet::<String>::new();

    for used in root_uses {
        if is_std_unit(used) {
            continue;
        }
        queue.push(canonical_unit_key(used));
    }

    while let Some(next) = queue.pop() {
        if !reachable.insert(next.clone()) {
            continue;
        }
        let Some(unit_file) = units.get(&next) else {
            return Err(unknown_unit_error(&next, units, "program"));
        };
        for used in &unit_file.unit.uses {
            if is_std_unit(used) {
                continue;
            }
            let key = canonical_unit_key(used);
            if !units.contains_key(&key) {
                let owner = qualified_id_to_string(&unit_file.unit.name);
                return Err(unknown_unit_error(&key, units, &format!("unit `{owner}`")));
            }
            queue.push(key);
        }
    }

    Ok(reachable)
}

pub(super) fn topo_sort_units(
    reachable: &HashSet<String>,
    units: &HashMap<String, UnitFile>,
) -> Result<Vec<String>, String> {
    let mut order = Vec::<String>::new();
    let mut state = HashMap::<String, VisitState>::new();
    let mut stack = Vec::<String>::new();

    for unit_key in reachable {
        topo_visit(
            unit_key, reachable, units, &mut state, &mut stack, &mut order,
        )?;
    }
    Ok(order)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Done,
}

fn topo_visit(
    key: &str,
    reachable: &HashSet<String>,
    units: &HashMap<String, UnitFile>,
    state: &mut HashMap<String, VisitState>,
    stack: &mut Vec<String>,
    order: &mut Vec<String>,
) -> Result<(), String> {
    match state.get(key) {
        Some(VisitState::Done) => return Ok(()),
        Some(VisitState::Visiting) => {
            let cycle_start = stack.iter().position(|item| item == key).unwrap_or(0);
            let cycle = stack[cycle_start..]
                .iter()
                .map(|unit_key| display_unit_key(unit_key))
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(format!(
                "Cyclic unit dependency detected: {cycle} -> {}.\n  help: Break the cycle by extracting shared declarations into a separate unit.",
                display_unit_key(key)
            ));
        }
        None => {}
    }

    state.insert(key.to_string(), VisitState::Visiting);
    stack.push(key.to_string());

    let Some(unit_file) = units.get(key) else {
        return Err(internal_link_error(
            key,
            "walking the topological dependency graph",
        ));
    };
    for used in &unit_file.unit.uses {
        if is_std_unit(used) {
            continue;
        }
        let dep_key = canonical_unit_key(used);
        if reachable.contains(&dep_key) {
            topo_visit(&dep_key, reachable, units, state, stack, order)?;
        }
    }

    stack.pop();
    state.insert(key.to_string(), VisitState::Done);
    order.push(key.to_string());
    Ok(())
}
