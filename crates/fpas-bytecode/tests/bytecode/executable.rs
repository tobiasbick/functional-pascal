use fpas_bytecode::{
    Constant, FunctionId, InstructionAddress, SharedFunction, SourceId, SourceMap, SourceRun,
    StringId, Value,
};
use std::sync::Arc;

use super::support::{all_opcodes_executable, minimal_executable};

#[test]
fn minimal_candidate_becomes_a_verified_executable() {
    let verified = minimal_executable().verify().expect("fixture must verify");
    assert_eq!(verified.executable().functions.len(), 1);
}

#[test]
fn verifier_accepts_a_fixture_covering_every_active_opcode() {
    let executable = all_opcodes_executable();
    assert_eq!(executable.code.len(), fpas_bytecode::Opcode::ALL.len() + 11);
    assert!(executable.verify().is_ok());
}

#[test]
fn sparse_source_map_uses_closest_preceding_run() {
    let map = SourceMap {
        sources: vec![StringId::new(0)],
        runs: vec![
            SourceRun {
                instruction_start: InstructionAddress::new(2),
                source: SourceId::new(0),
                line: 3,
                column: 4,
            },
            SourceRun {
                instruction_start: InstructionAddress::new(8),
                source: SourceId::new(0),
                line: 9,
                column: 1,
            },
        ],
    };
    assert_eq!(map.lookup(InstructionAddress::new(1)), None);
    assert_eq!(map.lookup(InstructionAddress::new(2)), Some(map.runs[0]));
    assert_eq!(map.lookup(InstructionAddress::new(7)), Some(map.runs[0]));
    assert_eq!(map.lookup(InstructionAddress::new(8)), Some(map.runs[1]));
}

#[test]
fn real_constants_preserve_nan_payload_and_signed_zero_identity() {
    assert_ne!(
        Constant::Real(0.0_f64.to_bits()),
        Constant::Real((-0.0_f64).to_bits())
    );
    assert_ne!(
        Constant::Real(0x7ff8_0000_0000_0001),
        Constant::Real(0x7ff8_0000_0000_0002)
    );
}

#[test]
fn function_targets_require_a_verified_table_entry() {
    let executable = minimal_executable().verify().expect("fixture must verify");
    let target = executable
        .function_target(FunctionId::new(0))
        .expect("verified entry");
    assert_eq!(target.function(), FunctionId::new(0));
    assert_eq!(target.name().as_ref(), "root");
    assert!(target.belongs_to(&executable));
    assert!(executable.function_target(FunctionId::new(1)).is_none());
    assert!(
        executable
            .function_target(FunctionId::new(u16::MAX))
            .is_none()
    );
}

#[test]
fn independently_verified_or_cloned_images_have_distinct_code_identities() {
    let executable = minimal_executable().verify().expect("fixture must verify");
    let target = executable
        .function_target(FunctionId::new(0))
        .expect("verified entry");
    let cloned = executable.clone();
    let reverified = executable
        .executable()
        .clone()
        .verify()
        .expect("equal bytes must verify");
    for independent in [&cloned, &reverified] {
        assert_eq!(&executable, independent);
        assert!(!target.belongs_to(independent));
        let foreign = independent
            .function_target(FunctionId::new(0))
            .expect("same numeric entry");
        assert_eq!(target.function(), foreign.function());
        assert!(!target.same_executable(&foreign));
        assert_ne!(target, foreign);
    }
}

#[test]
fn shared_images_and_function_copies_preserve_code_identity() {
    let executable = Arc::new(minimal_executable().verify().expect("fixture must verify"));
    let shared = Arc::clone(&executable);
    let target = executable
        .function_target(FunctionId::new(0))
        .expect("verified entry");
    let function = SharedFunction::unbound(target.clone(), vec![Value::Integer(42)]);
    let copy = function.clone();
    assert!(copy.target().belongs_to(&shared));
    assert_eq!(copy.target(), &target);
    assert_eq!(copy.function, target.function());
    assert!(Arc::ptr_eq(&copy.name, target.name()));
    assert_eq!(copy.captures, vec![Value::Integer(42)]);

    let bound = SharedFunction::bound(target.clone(), Value::Integer(7));
    assert_eq!(bound.target(), &target);
    assert_eq!(bound.bound_receiver, Some(Value::Integer(7)));
    assert!(bound.target().belongs_to(&shared));
}

#[test]
fn targets_surviving_an_image_do_not_match_a_reloaded_image() {
    let candidate = minimal_executable();
    let target = {
        let executable = candidate.clone().verify().expect("fixture must verify");
        executable
            .function_target(FunctionId::new(0))
            .expect("verified entry")
    };
    let reloaded = candidate.verify().expect("fixture must verify");
    assert_eq!(target.name().as_ref(), "root");
    assert!(!target.belongs_to(&reloaded));
}

#[test]
fn function_equality_includes_captures_domain_and_code_provenance() {
    let image = minimal_executable().verify().expect("fixture must verify");
    let target = image.function_target(FunctionId::new(0)).expect("entry");
    let owner = fpas_bytecode::MemoryDomain::new();
    let other = fpas_bytecode::MemoryDomain::new();
    let left = Value::function(target.clone(), vec![Value::Integer(1)]);
    let right = Value::function(target.clone(), vec![Value::Integer(1)]);
    let local =
        Value::task_owned_function(target.clone(), vec![Value::Integer(1)], owner.id().clone());
    let foreign =
        Value::task_owned_function(target.clone(), vec![Value::Integer(1)], other.id().clone());
    assert_eq!(left, right);
    assert_ne!(left, Value::function(target, vec![Value::Integer(2)]));
    assert_ne!(right, local);
    assert_ne!(local, foreign);
    let independent = image.clone();
    assert_ne!(
        left,
        Value::function(
            independent
                .function_target(FunctionId::new(0))
                .expect("entry"),
            vec![Value::Integer(1)]
        )
    );
}

#[test]
fn display_name_overrides_preserve_provenance_ownership_and_existing_copies() {
    let image = minimal_executable().verify().expect("fixture must verify");
    let target = image.function_target(FunctionId::new(0)).expect("entry");
    let domain = fpas_bytecode::MemoryDomain::new();
    let function = SharedFunction::task_owned(
        target.clone(),
        vec![Value::Integer(42)],
        domain.id().clone(),
    );
    let original = function.clone();
    let displayed = function.with_display_name("Root.Display");

    assert_eq!(displayed.name.as_ref(), "Root.Display");
    assert_eq!(original.name.as_ref(), "root");
    assert_eq!(displayed.target(), &target);
    assert!(displayed.target().belongs_to(&image));
    assert_eq!(displayed.target().name().as_ref(), "root");
    assert_eq!(displayed.function, original.function);
    assert_eq!(displayed.owner_domain, original.owner_domain);
    assert_eq!(displayed.task_bound, original.task_bound);
    assert_eq!(displayed.captures, original.captures);
}
