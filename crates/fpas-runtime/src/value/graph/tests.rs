use super::*;
use crate::{
    CodeRange, DebugType, Executable, FunctionFlags, FunctionId, FunctionInfo, FunctionTarget,
    Instruction, InstructionAddress, NO_REGISTER, Opcode, ReturnConvention, RuntimeEnumLayout,
    RuntimeRecordLayout, SharedEnum, SharedRecord, SourceId, SourceMap, SourceRun, StringId,
    StringTable, VariableReference,
};

mod inspection;
mod teardown;

pub(in crate::value) fn target() -> FunctionTarget {
    let image = Executable {
        code: vec![Instruction::abc(Opcode::Return, NO_REGISTER, 0, 0, 0).unwrap()],
        functions: vec![FunctionInfo {
            name: StringId::new(0),
            code: CodeRange::new(InstructionAddress::new(0), InstructionAddress::new(1)),
            arity: 0,
            capture_count: 0,
            register_count: 0,
            return_convention: ReturnConvention::Unit,
            flags: FunctionFlags::default(),
            debug: crate::FunctionDebugInfo::default(),
        }],
        constants: Vec::new(),
        strings: StringTable::new(vec!["work".into(), "graph.fpas".into()]),
        globals: Vec::new(),
        records: Vec::new(),
        enums: Vec::new(),
        enum_variants: Vec::new(),
        debug_types: vec![DebugType::Dynamic],
        distinct_types: Vec::new(),
        source_map: SourceMap {
            sources: vec![StringId::new(1)],
            runs: vec![SourceRun {
                instruction_start: InstructionAddress::new(0),
                source: SourceId::new(0),
                line: 1,
                column: 1,
            }],
        },
        entry: FunctionId::new(0),
    }
    .verify()
    .unwrap();
    image.function_target(FunctionId::new(0)).unwrap()
}

#[test]
fn every_aggregate_payload_and_dictionary_key_exposes_local_ownership() {
    let owner = MemoryDomain::new();
    let other = MemoryDomain::new();
    let cell = Value::Cell(owner.new_cell(Value::Integer(3)).unwrap());
    let variants = [
        Value::Array(vec![cell.clone()].into()),
        Value::dict(vec![(cell.clone(), Value::Unit)]),
        Value::dict(vec![(Value::Integer(1), cell.clone())]),
        Value::result_ok(cell.clone()),
        Value::result_error(cell.clone()),
        Value::option_some(cell.clone()),
        Value::Record(SharedRecord::new(
            Arc::new(RuntimeRecordLayout {
                record: crate::RecordTypeId::new(0),
                type_name: "Holder".into(),
                fields: vec!["Value".into()],
            }),
            vec![cell.clone()],
        )),
        Value::Enum(SharedEnum::new(
            Arc::new(RuntimeEnumLayout {
                error: None,
                enumeration: crate::EnumTypeId::new(0),
                variant_id: crate::EnumVariantId::new(0),
                type_name: "Choice".into(),
                variant: "Some".into(),
                fields: vec!["Value".into()],
            }),
            vec![cell.clone()],
        )),
        Value::function(target(), vec![cell.clone()]),
        Value::bound_function(target(), cell),
    ];
    for value in variants {
        let classification = classify_values(std::slice::from_ref(&value)).unwrap();
        assert!(!classification.is_transferable());
        assert!(classification.validate_domain(&owner).is_ok());
        assert_eq!(
            classification.validate_domain(&other),
            Err(OwnershipError::ForeignDomain)
        );
    }
}

#[test]
fn references_include_cell_and_global_owners_and_never_transfer() {
    let owner = MemoryDomain::new();
    let foreign = MemoryDomain::new();
    for root in [
        ReferenceRoot::Cell(owner.new_cell(Value::Integer(1)).unwrap()),
        ReferenceRoot::Global {
            slot: 0,
            owner: owner.id().clone(),
        },
    ] {
        let value = Value::Reference(Arc::new(VariableReference {
            root,
            path: Vec::new(),
        }));
        let classification = classify_values(std::slice::from_ref(&value)).unwrap();
        assert!(classification.has_references);
        assert!(!classification.is_transferable());
        assert_eq!(
            validate_local(&foreign, &value),
            Err(OwnershipError::ForeignDomain)
        );
    }
}

#[test]
fn callable_capability_and_code_identity_are_independent_of_cell_reachability() {
    let owner = MemoryDomain::new();
    let first = target();
    let second = target();
    let values = [
        Value::task_owned_function(first.clone(), vec![], owner.id().clone()),
        Value::function(second.clone(), vec![]),
    ];
    let classification = classify_values(&values).unwrap();
    assert!(classification.task_bound);
    assert!(!classification.is_transferable());
    assert_eq!(classification.owners.len(), 1);
    assert_eq!(classification.executable_identities.len(), 2);
    assert!(
        classification
            .executable_identities
            .contains(first.identity())
    );
    assert!(
        classification
            .executable_identities
            .contains(second.identity())
    );
    assert!(
        classify_values(&[Value::function(first, vec![Value::Integer(2)])])
            .unwrap()
            .is_transferable()
    );
}

#[test]
fn summarized_transfer_never_admits_what_classification_rejects() {
    let owner = MemoryDomain::new();
    let cell = Value::Cell(owner.new_cell(Value::Integer(1)).unwrap());
    let candidates = [
        Value::Integer(1),
        Value::Array(vec![Value::Integer(2)].into()),
        Value::function(target(), vec![Value::Integer(3)]),
        Value::bound_function(target(), Value::Integer(4)),
        Value::function(target(), vec![cell.clone()]),
        Value::bound_function(target(), cell.clone()),
        Value::function(target(), vec![Value::function(target(), vec![])]),
        Value::task_owned_function(target(), vec![], owner.id().clone()),
        Value::option_some(cell),
    ];
    for value in &candidates {
        let values = std::slice::from_ref(value);
        if summarized_transferable(values) {
            assert!(classify_values(values).unwrap().is_transferable());
        }
    }
    assert!(summarized_transferable(&candidates[..4]));
    assert!(!summarized_transferable(&candidates[4..6]));
    assert!(!summarized_transferable(&candidates[7..]));
}

#[test]
fn cyclic_closure_graphs_finish_and_preserve_aliases() {
    let owner = MemoryDomain::new();
    let cell = owner.new_cell(Value::Unit).unwrap();
    cell.replace(
        &owner,
        Value::function(
            target(),
            vec![Value::option_some(Value::Cell(cell.clone()))],
        ),
    )
    .unwrap();
    let graph = classify_values(&[Value::Cell(cell.clone())]).unwrap();
    assert_eq!(graph.owners.len(), 1);
    assert!(graph.owners.contains(owner.id()));
    assert_eq!(graph.executable_identities.len(), 1);
    let weak = cell.downgrade();
    cell.replace(&owner, Value::Unit).unwrap();
    drop(cell);
    assert!(weak.upgrade().is_none());
}

#[test]
fn deeply_wrapped_payloads_are_walked_and_released_without_recursive_cloning() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let owner = MemoryDomain::new();
            let mut value = Value::Integer(1);
            for _ in 0..100_000 {
                value = Value::option_some(value);
            }
            let cell = owner.new_cell(value).unwrap();
            let snapshot = cell.read_snapshot(&owner).unwrap();
            assert_eq!(
                classify_values(&[Value::Cell(cell.clone())])
                    .unwrap()
                    .owners
                    .len(),
                1
            );
            cell.replace(&owner, Value::Unit).unwrap();
            drop(snapshot);
            drop(cell);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn shared_dag_nodes_are_not_expanded_exponentially() {
    let owner = MemoryDomain::new();
    let mut value = Value::Integer(1);
    for _ in 0..1024 {
        value = Value::Array(vec![value.clone(), value].into());
    }
    assert!(
        classify_values(std::slice::from_ref(&value))
            .unwrap()
            .is_transferable()
    );
    drop(owner.new_cell(value).unwrap());
    assert_eq!(owner.registered_count(), 0);
}

#[test]
fn primitive_arrays_do_not_enter_the_graph_traversal_queue() {
    let mut values = crate::SharedArray::from(vec![Value::Integer(1); 4096]);
    values.push(Value::Str("tail".into()));
    values.replace_element(0, Value::OptionNone);
    let mut walker = Walker::default();
    walker.enqueue(&Value::Array(values)).unwrap();
    assert!(walker.pending.is_empty());
    assert!(walker.finish().unwrap().is_transferable());
}

#[test]
fn arrays_of_records_keep_constant_time_classification_after_nested_updates() {
    let record = SharedRecord::new(
        Arc::new(RuntimeRecordLayout {
            record: crate::RecordTypeId::new(0),
            type_name: "CellStyle".into(),
            fields: vec!["Style".into()],
        }),
        vec![Value::option_some(Value::Integer(1))],
    );
    let mut array = crate::SharedArray::from(vec![Value::Record(record); 4096]);
    for index in 0..array.len() {
        array.update_element(index, |value| {
            let Value::Record(record) = value else {
                unreachable!()
            };
            record.replace_field(0, Value::option_some(Value::Integer(2)));
        });
        let mut walker = Walker::default();
        walker.enqueue(&Value::Array(array.clone())).unwrap();
        assert!(walker.pending.is_empty());
    }
    let function = target();
    array.update_element(0, |value| {
        let Value::Record(record) = value else {
            unreachable!()
        };
        record.replace_field(0, Value::function(function.clone(), Vec::new()));
    });
    let classification = classify_values(&[Value::Array(array)]).unwrap();
    assert!(
        classification
            .executable_identities
            .contains(function.identity())
    );
}

#[test]
fn mutated_arrays_expose_inserted_owners_references_and_callables() {
    let owner = MemoryDomain::new();
    let foreign = MemoryDomain::new();
    let function = target();
    let candidates = [
        Value::Cell(owner.new_cell(Value::Unit).unwrap()),
        Value::Reference(Arc::new(VariableReference {
            root: ReferenceRoot::Global {
                slot: 0,
                owner: owner.id().clone(),
            },
            path: Vec::new(),
        })),
        Value::task_owned_function(function.clone(), Vec::new(), owner.id().clone()),
        Value::function(function.clone(), Vec::new()),
    ];
    for value in candidates {
        assert!(!crate::SharedArray::from(vec![value.clone()]).is_plain_data());
        for method in 0..3 {
            let original = crate::SharedArray::from(vec![Value::Unit]);
            let mut mutated = original.clone();
            match method {
                0 => mutated.push(value.clone()),
                1 => {
                    mutated.replace_element(0, value.clone());
                }
                _ => mutated[0] = value.clone(),
            }
            assert!(original.is_plain_data());
            assert!(!mutated.is_plain_data());
            let classification = classify_values(&[Value::Array(mutated)]).unwrap();
            match &value {
                Value::Cell(_) | Value::Reference(_) => {
                    assert_eq!(
                        classification.validate_domain(&foreign),
                        Err(OwnershipError::ForeignDomain)
                    );
                    assert_eq!(
                        classification.has_references,
                        matches!(value, Value::Reference(_))
                    );
                }
                Value::Function(callable) => {
                    assert_eq!(classification.task_bound, callable.task_bound);
                    assert_eq!(
                        classification.owners.contains(owner.id()),
                        callable.owner_domain.is_some()
                    );
                    assert!(
                        classification
                            .executable_identities
                            .contains(function.identity())
                    );
                }
                _ => unreachable!(),
            }
        }
    }
}
