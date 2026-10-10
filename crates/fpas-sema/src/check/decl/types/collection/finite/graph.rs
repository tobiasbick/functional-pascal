//! Monotone finite-value constraints and cycle witnesses for a shared nominal graph.
//!
//! **Documentation:** `docs/pascal/language/types/declaration-order.md`

use super::Checker;
use crate::types::Ty;
use std::collections::{HashMap, HashSet, VecDeque};

#[cfg(test)]
mod tests;

#[derive(Clone, Copy)]
enum Requirement {
    All,
    Any,
}

struct Dependency {
    target: usize,
    label: Option<String>,
}

struct Node {
    name: Option<String>,
    requirement: Requirement,
    dependencies: Vec<Dependency>,
}

/// Construction constraints, with one node per canonical record or enum identity.
/// See `docs/pascal/language/types/declaration-order.md`.
pub(super) struct FiniteGraph {
    nodes: Vec<Node>,
    nominal: HashMap<String, usize>,
    pending: Vec<(usize, Ty)>,
    #[cfg(test)]
    expanded_nominal: usize,
}

/// Least fixed point of types whose required values have a terminating construction.
/// See `docs/pascal/language/types/declaration-order.md`.
pub(super) struct FiniteValues {
    finite: Vec<bool>,
    #[cfg(test)]
    propagated_edges: usize,
}

impl FiniteValues {
    /// Whether this construction constraint admits a finite value.
    pub(super) fn is_finite(&self, node: usize) -> bool {
        self.finite[node]
    }
}

impl FiniteGraph {
    /// Collect reachable stored payloads once, sharing nominal definitions across roots.
    /// See `docs/pascal/language/types/declaration-order.md`.
    pub(super) fn new<'a>(
        checker: &Checker,
        types: impl Iterator<Item = &'a Ty>,
    ) -> (Self, Vec<usize>) {
        let mut graph = Self {
            nodes: Vec::new(),
            nominal: HashMap::new(),
            pending: Vec::new(),
            #[cfg(test)]
            expanded_nominal: 0,
        };
        // Scalars, empty containers, signatures and handles share a terminating seed.
        graph.add_node(None, Requirement::All);
        let roots = types.map(|ty| graph.type_node(checker, ty)).collect();
        while let Some((node, ty)) = graph.pending.pop() {
            #[cfg(test)]
            if matches!(ty, Ty::Record(_) | Ty::Enum(_)) {
                graph.expanded_nominal += 1;
            }
            let dependencies = match ty {
                Ty::Record(record) => graph.fields(checker, &record.name, &record.fields),
                Ty::Enum(enumeration) => enumeration
                    .variants
                    .iter()
                    .map(|variant| {
                        let alternative = graph.add_node(None, Requirement::All);
                        graph.nodes[alternative].dependencies = graph.fields(
                            checker,
                            &format!("{}.{}", enumeration.name, variant.name),
                            &variant.fields,
                        );
                        Dependency {
                            target: alternative,
                            label: None,
                        }
                    })
                    .collect(),
                Ty::Result(ok, error) => [&*ok, &*error]
                    .into_iter()
                    .map(|ty| Dependency {
                        target: graph.type_node(checker, ty),
                        label: None,
                    })
                    .collect(),
                _ => unreachable!("only nominal and Result nodes are queued"),
            };
            graph.nodes[node].dependencies = dependencies;
        }
        (graph, roots)
    }

    fn add_node(&mut self, name: Option<String>, requirement: Requirement) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node {
            name,
            requirement,
            dependencies: Vec::new(),
        });
        id
    }

    fn type_node(&mut self, checker: &Checker, ty: &Ty) -> usize {
        let resolved = checker.resolve_visible_type(ty);
        let (name, requirement) = match &resolved {
            Ty::Record(_) => (Some(resolved.to_string()), Requirement::All),
            Ty::Enum(_) => (Some(resolved.to_string()), Requirement::Any),
            Ty::Result(..) => (None, Requirement::Any),
            _ => return 0,
        };
        let key = name.as_ref().map(|name| name.to_ascii_lowercase());
        if let Some(node) = key.as_ref().and_then(|key| self.nominal.get(key)) {
            return *node;
        }
        let node = self.add_node(name, requirement);
        if let Some(key) = key {
            self.nominal.insert(key, node);
        }
        self.pending.push((node, resolved));
        node
    }

    fn fields(
        &mut self,
        checker: &Checker,
        owner: &str,
        fields: &[(String, Ty)],
    ) -> Vec<Dependency> {
        fields
            .iter()
            .map(|(field, ty)| Dependency {
                target: self.type_node(checker, ty),
                label: Some(format!("{owner}.{field}")),
            })
            .collect()
    }

    /// Propagate each finite node once; each dependency receives at most one notification.
    /// See `docs/pascal/language/types/declaration-order.md`.
    pub(super) fn solve(&self) -> FiniteValues {
        let mut dependents = vec![Vec::new(); self.nodes.len()];
        let mut remaining: Vec<_> = self
            .nodes
            .iter()
            .map(|node| node.dependencies.len())
            .collect();
        for (parent, node) in self.nodes.iter().enumerate() {
            for dependency in &node.dependencies {
                dependents[dependency.target].push(parent);
            }
        }
        let mut values = FiniteValues {
            finite: vec![false; self.nodes.len()],
            #[cfg(test)]
            propagated_edges: 0,
        };
        let mut queue = VecDeque::new();
        for (id, node) in self.nodes.iter().enumerate() {
            if matches!(node.requirement, Requirement::All) && remaining[id] == 0 {
                values.finite[id] = true;
                queue.push_back(id);
            }
        }
        while let Some(finite) = queue.pop_front() {
            for &parent in &dependents[finite] {
                #[cfg(test)]
                {
                    values.propagated_edges += 1;
                }
                if values.finite[parent] {
                    continue;
                }
                let ready = match self.nodes[parent].requirement {
                    Requirement::All => {
                        remaining[parent] -= 1;
                        remaining[parent] == 0
                    }
                    Requirement::Any => true,
                };
                if ready {
                    values.finite[parent] = true;
                    queue.push_back(parent);
                }
            }
        }
        values
    }

    /// Follow non-finite dependencies to a repeated type without expanding alternatives.
    /// See `docs/pascal/language/types/declaration-order.md`.
    pub(super) fn cycle(&self, root: usize, values: &FiniteValues) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        let mut current = root;
        loop {
            let node = &self.nodes[current];
            if !visited.insert(current) {
                if let Some(name) = &node.name {
                    path.push(name.clone());
                }
                return path;
            }
            let Some(dependency) = node
                .dependencies
                .iter()
                .find(|dependency| !values.is_finite(dependency.target))
            else {
                return Vec::new();
            };
            if let Some(label) = &dependency.label {
                path.push(label.clone());
            }
            current = dependency.target;
        }
    }
}
