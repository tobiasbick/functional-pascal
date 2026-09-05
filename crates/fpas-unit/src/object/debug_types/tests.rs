//! Shared subgraphs, cycle rejection, and path-sensitive depth limits.

use super::{ObjectDebugType, ObjectError, validate_debug_types};

fn validate(types: &[ObjectDebugType]) -> Result<(), ObjectError> {
    validate_debug_types(types, &[], &[], &[])
}

fn chain(depth: u32) -> Vec<ObjectDebugType> {
    let mut types = vec![ObjectDebugType::Integer];
    types.extend((0..depth).map(ObjectDebugType::Array));
    types
}

#[test]
fn accepts_shared_subgraphs_in_both_reference_orders() {
    let mut types = vec![ObjectDebugType::Integer];
    for child in 0..16 {
        types.push(ObjectDebugType::Dictionary {
            key: child,
            value: child,
        });
    }
    assert_eq!(validate(&types), Ok(()));
    types.reverse();
    for ty in &mut types {
        if let ObjectDebugType::Dictionary { key, value } = ty {
            *key = 16 - *key;
            *value = 16 - *value;
        }
    }
    assert_eq!(validate(&types), Ok(()));
}

#[test]
fn cached_subgraph_depth_still_rejects_a_longer_incoming_path() {
    assert_eq!(validate(&chain(64)), Ok(()));
    assert_eq!(
        validate(&chain(65)),
        Err(ObjectError::InvalidTableReference("debug type graph"))
    );
}

#[test]
fn forward_references_preserve_the_exact_depth_boundary() {
    for depth in [64, 65] {
        let mut types = (1..=depth).map(ObjectDebugType::Array).collect::<Vec<_>>();
        types.push(ObjectDebugType::Integer);
        assert_eq!(validate(&types).is_ok(), depth == 64);
    }
}

#[test]
fn rejects_cycles_after_visiting_a_valid_shared_branch() {
    for types in [
        vec![ObjectDebugType::Array(0)],
        vec![
            ObjectDebugType::Integer,
            ObjectDebugType::Dictionary { key: 0, value: 2 },
            ObjectDebugType::Array(1),
        ],
    ] {
        assert_eq!(
            validate(&types),
            Err(ObjectError::InvalidTableReference("debug type graph"))
        );
    }
}

#[test]
fn rejects_missing_children_and_excessive_function_parameters() {
    assert_eq!(
        validate(&[ObjectDebugType::Array(u32::MAX)]),
        Err(ObjectError::InvalidTableReference("debug type child"))
    );
    let types = [ObjectDebugType::Function {
        parameters: vec![0; fpas_bytecode::limits::MAX_CALL_ARGUMENTS + 1],
        result: 0,
    }];
    assert_eq!(
        validate(&types),
        Err(ObjectError::InvalidTableReference(
            "debug function parameter count"
        ))
    );
    assert_eq!(validate(&[]), Ok(()));
}

#[test]
fn completed_siblings_are_reusable_without_being_mistaken_for_cycles() {
    let types = [
        ObjectDebugType::Integer,
        ObjectDebugType::Function {
            parameters: vec![0, 0, 0],
            result: 0,
        },
        ObjectDebugType::Result { ok: 1, error: 1 },
    ];
    assert_eq!(validate(&types), Ok(()));
}
