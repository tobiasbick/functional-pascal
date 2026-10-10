//! Deterministic work bounds for shared finite and recursive type graphs.

use super::*;
use crate::scope::{Symbol, SymbolKind};
use crate::types::{EnumTy, EnumVariantTy, RecordTy};
use std::sync::Arc;

fn record(name: &str, fields: Vec<(String, Ty)>) -> Ty {
    Ty::Record(Arc::new(RecordTy {
        type_params: Vec::new(),
        type_args: Vec::new(),
        name: name.into(),
        fields,
        owner_unit: None,
        private_members: Vec::new(),
        methods: Vec::new(),
        static_functions: Vec::new(),
        static_procedures: Vec::new(),
    }))
}

fn checker(types: &[Ty]) -> Checker {
    let mut checker = Checker::new();
    for ty in types {
        let name = match ty {
            Ty::Record(record) => &record.name,
            Ty::Enum(enumeration) => &enumeration.name,
            _ => unreachable!(),
        };
        assert!(checker.scopes.define(
            name,
            Symbol {
                ty: ty.clone(),
                mutable: false,
                kind: SymbolKind::Type,
                constant: None,
                task_bound: false
            }
        ));
    }
    checker
}

#[test]
fn shared_acyclic_records_expand_once_across_all_roots() {
    for depth in [22, 64, 256] {
        let mut types = Vec::new();
        for i in 0..depth {
            types.push(record(
                &format!("R{i}"),
                vec![
                    ("Left".into(), Ty::Named(format!("R{}", i + 1))),
                    ("Right".into(), Ty::Named(format!("r{}", i + 1))),
                ],
            ));
        }
        types.push(record(
            &format!("R{depth}"),
            vec![("Value".into(), Ty::Integer)],
        ));
        let checker = checker(&types);
        let (graph, roots) = FiniteGraph::new(&checker, types.iter());
        let values = graph.solve();
        assert!(roots.iter().all(|&root| values.is_finite(root)));
        assert_eq!(graph.nominal.len(), types.len());
        assert_eq!(graph.expanded_nominal, types.len());
        assert_eq!(graph.nodes.len(), types.len() + 1);
        assert_eq!(values.propagated_edges, 2 * depth + 1);
    }
}

#[test]
fn shared_recursive_alternatives_terminate_from_a_single_seed() {
    let count = 256;
    let types: Vec<_> = (0..count)
        .map(|i| {
            Ty::Enum(Arc::new(EnumTy {
                name: format!("E{i}"),
                type_params: Vec::new(),
                type_args: Vec::new(),
                variants: vec![
                    EnumVariantTy {
                        name: "More".into(),
                        fields: vec![
                            ("Left".into(), Ty::Named(format!("E{}", (i + 1) % count))),
                            ("Right".into(), Ty::Named(format!("E{}", (i + 1) % count))),
                        ],
                        backing_value: None,
                    },
                    EnumVariantTy {
                        name: "Stop".into(),
                        fields: if i == count - 1 {
                            Vec::new()
                        } else {
                            vec![("Next".into(), Ty::Named(format!("E{}", i + 1)))]
                        },
                        backing_value: None,
                    },
                ],
            }))
        })
        .collect();
    let checker = checker(&types);
    let (graph, roots) = FiniteGraph::new(&checker, types.iter());
    let values = graph.solve();
    assert!(roots.iter().all(|&root| values.is_finite(root)));
    assert_eq!(graph.nominal.len(), count);
    assert_eq!(graph.expanded_nominal, count);
    let edges: usize = graph.nodes.iter().map(|node| node.dependencies.len()).sum();
    assert_eq!(edges, 5 * count - 1);
    assert_eq!(values.propagated_edges, edges);
}

#[test]
fn repeated_non_terminating_alternatives_do_not_expand_failure_paths() {
    let count = 128;
    let types: Vec<_> = (0..count)
        .map(|i| {
            Ty::Enum(Arc::new(EnumTy {
                name: format!("E{i}"),
                type_params: Vec::new(),
                type_args: Vec::new(),
                variants: ["Left", "Right"]
                    .into_iter()
                    .map(|name| EnumVariantTy {
                        name: name.into(),
                        fields: vec![("Next".into(), Ty::Named(format!("E{}", (i + 1) % count)))],
                        backing_value: None,
                    })
                    .collect(),
            }))
        })
        .collect();
    let checker = checker(&types);
    let (graph, roots) = FiniteGraph::new(&checker, types.iter());
    let values = graph.solve();
    assert!(roots.iter().all(|&root| !values.is_finite(root)));
    assert_eq!(graph.nominal.len(), count);
    assert_eq!(graph.expanded_nominal, count);
    assert_eq!(graph.nodes.len(), 3 * count + 1);
    assert_eq!(values.propagated_edges, 0);
    for root in roots {
        let cycle = graph.cycle(root, &values);
        assert_eq!(cycle.len(), count + 1);
        assert!(cycle.last().is_some_and(|name| name.starts_with('E')));
    }
}
