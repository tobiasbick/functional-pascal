//! Shared type subgraphs and deterministic executable validation failures.

use fpas_bytecode::{DebugType, DebugTypeId, ValidationErrorKind};

use super::support::minimal_executable;

fn validate(types: Vec<DebugType>) -> Result<(), ValidationErrorKind> {
    let mut executable = minimal_executable();
    executable.debug_types = types;
    executable.verify().map(|_| ()).map_err(|error| error.kind)
}

#[test]
fn shared_subgraphs_validate_in_both_reference_orders() {
    for forward in [false, true] {
        let mut types = vec![DebugType::Integer];
        for child in 0..16 {
            let id = DebugTypeId::new(child);
            types.push(DebugType::Dictionary { key: id, value: id });
        }
        if forward {
            types.reverse();
            for ty in &mut types {
                if let DebugType::Dictionary { key, value } = ty {
                    *key = DebugTypeId::new(16 - key.get());
                    *value = DebugTypeId::new(16 - value.get());
                }
            }
        }
        assert_eq!(validate(types), Ok(()));
    }
}

#[test]
fn shared_suffix_preserves_depth_boundary_and_first_excess_depth() {
    for forward in [false, true] {
        for depth in [64, 65, 70] {
            let types = if forward {
                (1..=depth)
                    .map(|id| DebugType::Array(DebugTypeId::new(id)))
                    .chain(std::iter::once(DebugType::Integer))
                    .collect()
            } else {
                std::iter::once(DebugType::Integer)
                    .chain((0..depth).map(|id| DebugType::Array(DebugTypeId::new(id))))
                    .collect()
            };
            let expected = if depth == 64 {
                Ok(())
            } else {
                Err(ValidationErrorKind::DebugTypeDepth {
                    actual: 65,
                    maximum: 64,
                })
            };
            assert_eq!(validate(types), expected);
        }
    }

    let mut types = (1..=64)
        .map(|id| DebugType::Array(DebugTypeId::new(id)))
        .chain(std::iter::once(DebugType::Integer))
        .collect::<Vec<_>>();
    types.extend((66..=70).map(|id| DebugType::Array(DebugTypeId::new(id))));
    types.push(DebugType::Array(DebugTypeId::new(0)));
    assert_eq!(
        validate(types),
        Err(ValidationErrorKind::DebugTypeDepth {
            actual: 65,
            maximum: 64,
        })
    );
}

#[test]
fn cycle_after_a_completed_shared_child_keeps_its_node_id() {
    assert_eq!(
        validate(vec![
            DebugType::Integer,
            DebugType::Dictionary {
                key: DebugTypeId::new(0),
                value: DebugTypeId::new(2)
            },
            DebugType::Array(DebugTypeId::new(1)),
        ]),
        Err(ValidationErrorKind::DebugTypeCycle { actual: 1 })
    );
}

#[test]
fn invalid_forward_child_keeps_the_traversal_diagnostic() {
    assert_eq!(
        validate(vec![
            DebugType::Array(DebugTypeId::new(1)),
            DebugType::Array(DebugTypeId::new(u32::MAX)),
        ]),
        Err(ValidationErrorKind::TableReference {
            table: "debug types",
            operand: "debug type",
            actual: u64::from(u32::MAX),
            length: 2,
        })
    );
}

#[test]
fn traversal_failure_still_precedes_a_later_nodes_shape_failure() {
    assert_eq!(
        validate(vec![
            DebugType::Array(DebugTypeId::new(1)),
            DebugType::Function {
                parameters: vec![
                    DebugTypeId::new(0);
                    fpas_bytecode::limits::MAX_CALL_ARGUMENTS + 1
                ],
                result: DebugTypeId::new(0),
            },
        ]),
        Err(ValidationErrorKind::DebugTypeCycle { actual: 0 })
    );
}

#[test]
fn empty_graph_and_repeated_function_children_remain_valid() {
    assert_eq!(validate(vec![]), Ok(()));
    assert_eq!(
        validate(vec![
            DebugType::Integer,
            DebugType::Function {
                parameters: vec![DebugTypeId::new(0); 4],
                result: DebugTypeId::new(0)
            },
        ]),
        Ok(())
    );
}
