//! Stable topological ordering for reachable source units.

use std::collections::{HashMap, HashSet};

use super::model::{ResolvedUnitGraph, UnitGraph};
use super::{canonical_unit_key, display_unit_key, internal_graph_error, is_intrinsic_std_unit};

pub(super) fn resolve_order(
    reachable: &HashSet<String>,
    graph: &UnitGraph,
) -> Result<ResolvedUnitGraph, String> {
    let mut order = Vec::<String>::new();
    let mut state = HashMap::<String, VisitState>::new();
    let mut stack = Vec::<String>::new();

    for unit_key in sorted_reachable_unit_keys(reachable) {
        topo_visit(
            &unit_key, reachable, graph, &mut state, &mut stack, &mut order,
        )?;
    }
    Ok(ResolvedUnitGraph::new(order))
}

fn sorted_reachable_unit_keys(reachable: &HashSet<String>) -> Vec<String> {
    let mut unit_keys = reachable.iter().cloned().collect::<Vec<_>>();
    unit_keys.sort();
    unit_keys
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Visiting,
    Done,
}

fn topo_visit(
    key: &str,
    reachable: &HashSet<String>,
    graph: &UnitGraph,
    state: &mut HashMap<String, VisitState>,
    stack: &mut Vec<String>,
    order: &mut Vec<String>,
) -> Result<(), String> {
    match state.get(key) {
        Some(VisitState::Done) => return Ok(()),
        Some(VisitState::Visiting) => {
            let Some(cycle_start) = stack.iter().position(|item| item == key) else {
                return Err(internal_graph_error(
                    key,
                    "reporting a cyclic dependency path",
                ));
            };
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

    let Some(node) = graph.get(key) else {
        return Err(internal_graph_error(
            key,
            "walking the topological dependency graph",
        ));
    };
    for used in node.direct_uses() {
        if is_intrinsic_std_unit(used, graph) {
            continue;
        }
        let dependency_key = canonical_unit_key(used);
        if reachable.contains(&dependency_key) {
            topo_visit(&dependency_key, reachable, graph, state, stack, order)?;
        }
    }

    stack.pop();
    state.insert(key.to_string(), VisitState::Done);
    order.push(key.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::sorted_reachable_unit_keys;

    #[test]
    fn reachable_keys_are_sorted_lexically() {
        let reachable = HashSet::from([
            "app.beta".to_string(),
            "app.alpha".to_string(),
            "app.gamma".to_string(),
        ]);

        assert_eq!(
            sorted_reachable_unit_keys(&reachable),
            vec![
                "app.alpha".to_string(),
                "app.beta".to_string(),
                "app.gamma".to_string(),
            ]
        );
    }
}
