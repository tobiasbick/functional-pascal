//! Atomic case claims, resource limits, and task/VM ownership regressions.

use super::*;
use fpas_bytecode::{FunctionId, FunctionTarget, MemoryDomain};
use std::sync::OnceLock;

fn target() -> FunctionTarget {
    static TARGET: OnceLock<FunctionTarget> = OnceLock::new();
    TARGET
        .get_or_init(|| {
            let (program, errors) = fpas_parser::parse("program Callback; begin end.");
            assert!(errors.is_empty(), "{errors:?}");
            fpas_compiler::compile(&program)
                .expect("callback fixture")
                .function_target(FunctionId::new(0))
                .expect("callback target")
        })
        .clone()
}

fn case() -> WaitCase {
    WaitCase {
        roots: None,
        source: CaseSource::Timer(0),
        callback: SharedFunction::unbound(target(), vec![]),
    }
}

#[test]
fn invalid_or_duplicate_input_never_partially_claims_cases() {
    let registry = CaseRegistry::default();
    let id = registry.create(7, case).unwrap();
    assert!(registry.claim(7, &[id, 0]).is_err());
    assert!(registry.claim(7, &[id, id]).is_err());
    assert_eq!(registry.claim(7, &[id]).unwrap().len(), 1);
    assert!(registry.claim(7, &[id]).is_err());
    assert!(!registry.close(7, id).unwrap());
}

#[test]
fn case_ownership_cannot_cross_tasks_or_vm_instances() {
    let first = CaseRegistry::default();
    let second = CaseRegistry::default();
    let a = first.create(1, case).unwrap();
    let b = second.create(1, case).unwrap();
    assert_ne!(a, b);
    assert!(second.claim(1, &[a]).is_err());
    assert!(first.claim(2, &[a]).is_err());
    assert!(first.close(2, a).is_err());
    assert!(first.close(1, a).unwrap());
    assert!(second.close(1, b).unwrap());
}

#[test]
fn capacity_is_checked_before_retaining_user_values_and_close_releases_capacity() {
    let registry = CaseRegistry::default();
    let ids: Vec<_> = (0..MAX_UNUSED)
        .map(|_| registry.create(0, case).unwrap())
        .collect();
    assert!(
        registry
            .create(0, || panic!("must reject before retaining values"))
            .is_err()
    );
    assert!(registry.claim(0, &[]).is_err());
    assert!(registry.claim(0, &ids[..MAX_CASES + 1]).is_err());
    assert!(registry.close(0, ids[0]).unwrap());
    assert!(registry.create(0, case).is_ok());
}

#[test]
fn repeated_claims_and_closes_do_not_accumulate_entries() {
    let registry = CaseRegistry::default();
    for _ in 0..10_000 {
        let first = registry.create(0, case).unwrap();
        let second = registry.create(0, case).unwrap();
        drop(registry.claim(0, &[first]).unwrap());
        assert!(registry.close(0, second).unwrap());
        assert!(registry.entries.lock().unwrap().cases.is_empty());
    }
}

#[test]
fn close_claim_and_registry_teardown_release_callback_captures() {
    for path in 0..3 {
        let registry = CaseRegistry::default();
        let domain = MemoryDomain::new();
        let captured = domain.new_cell(Value::Integer(42)).expect("owned capture");
        let weak = captured.downgrade();
        let id = registry
            .create(0, || WaitCase {
                roots: None,
                source: CaseSource::Timer(0),
                callback: SharedFunction::task_owned(
                    target(),
                    vec![Value::Cell(captured)],
                    domain.id().clone(),
                ),
            })
            .unwrap();
        assert!(weak.upgrade().is_some());
        match path {
            0 => {
                registry.close(0, id).unwrap();
            }
            1 => {
                drop(registry.claim(0, &[id]).unwrap());
            }
            _ => {
                drop(registry);
            }
        }
        assert!(
            weak.upgrade().is_none(),
            "callback retained after exit path {path}"
        );
    }
}

#[test]
fn invalid_send_payload_is_rejected_before_retaining_its_local_callback() {
    let registry = CaseRegistry::default();
    let domain = MemoryDomain::new();
    let payload = domain.new_cell(Value::Integer(1)).expect("payload");
    for value in [
        Value::Cell(payload.clone()),
        Value::option_some(Value::Array(vec![Value::Cell(payload.clone())].into())),
    ] {
        let captured = domain
            .new_cell(Value::Integer(2))
            .expect("callback capture");
        let weak = captured.downgrade();
        assert!(
            registry
                .create(7, || WaitCase {
                    roots: None,
                    source: CaseSource::Send(0, value),
                    callback: SharedFunction::task_owned(
                        target(),
                        vec![Value::Cell(captured)],
                        domain.id().clone()
                    ),
                })
                .is_err()
        );
        assert!(
            weak.upgrade().is_none(),
            "failed retention releases callback"
        );
        assert!(registry.entries.lock().unwrap().cases.is_empty());
    }
}

#[test]
fn owner_cleanup_releases_only_unused_cases_and_keeps_retry_identities_fresh() {
    let registry = CaseRegistry::default();
    let domain = MemoryDomain::new();
    let create_owned = |owner| {
        let captured = domain.new_cell(Value::Integer(42)).expect("capture");
        let weak = captured.downgrade();
        let id = registry
            .create(owner, || WaitCase {
                roots: None,
                source: CaseSource::Timer(0),
                callback: SharedFunction::task_owned(
                    target(),
                    vec![Value::Cell(captured)],
                    domain.id().clone(),
                ),
            })
            .expect("case");
        (id, weak)
    };
    let (unused, unused_capture) = create_owned(7);
    let (selected, selected_capture) = create_owned(7);
    let (other, other_capture) = create_owned(8);
    let claimed = registry.claim(7, &[selected]).expect("claim");
    assert_eq!(registry.close_owner(7), 1);
    assert!(unused_capture.upgrade().is_none());
    assert!(
        selected_capture.upgrade().is_some(),
        "claimed case belongs to Select"
    );
    assert!(other_capture.upgrade().is_some());
    assert!(!registry.close(7, unused).expect("already closed"));
    assert!(registry.claim(7, &[unused]).is_err());
    assert!(
        registry.close(7, other).is_err(),
        "active wrong owner still rejects"
    );
    assert!(registry.close(7, 0).is_err(), "invalid kind still rejects");
    let (retry, retry_capture) = create_owned(7);
    assert!(retry > unused && retry > selected && retry > other);
    assert_eq!(registry.close_all(), 2);
    assert!(other_capture.upgrade().is_none());
    assert!(retry_capture.upgrade().is_none());
    assert!(selected_capture.upgrade().is_some());
    assert_eq!(registry.close_all(), 0);
    assert!(
        registry
            .create(7, || panic!("closed admission must not construct a case"))
            .is_err()
    );
    drop(claimed);
    assert!(selected_capture.upgrade().is_none());
}

#[test]
fn shutdown_during_case_construction_prevents_late_admission() {
    let registry = CaseRegistry::default();
    let domain = MemoryDomain::new();
    let captured = domain.new_cell(Value::Integer(42)).expect("capture");
    let weak = captured.downgrade();
    let result = registry.create(7, || {
        assert_eq!(registry.close_all(), 0);
        WaitCase {
            roots: None,
            source: CaseSource::Timer(0),
            callback: SharedFunction::task_owned(
                target(),
                vec![Value::Cell(captured)],
                domain.id().clone(),
            ),
        }
    });
    assert!(result.is_err());
    assert!(weak.upgrade().is_none());
    assert!(registry.entries.lock().unwrap().cases.is_empty());
}

#[test]
fn rejecting_deep_send_cases_releases_payload_and_callback_on_a_small_stack() {
    let target = target();
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(move || {
            let registry = CaseRegistry::default();
            let domain = MemoryDomain::new();
            for function_captures in [false, true] {
                let cell = domain.new_cell(Value::Integer(3)).expect("payload cell");
                let payload_weak = cell.downgrade();
                let mut payload = Value::Cell(cell);
                for _ in 0..30_000 {
                    payload = if function_captures {
                        Value::function(target.clone(), vec![payload])
                    } else {
                        Value::option_some(payload)
                    };
                }
                let captured = domain.new_cell(Value::Integer(2)).expect("callback cell");
                let callback_weak = captured.downgrade();
                assert!(
                    registry
                        .create(7, || WaitCase {
                            roots: None,
                            source: CaseSource::Send(0, payload),
                            callback: SharedFunction::task_owned(
                                target.clone(),
                                vec![Value::Cell(captured)],
                                domain.id().clone(),
                            ),
                        })
                        .is_err()
                );
                assert!(payload_weak.upgrade().is_none(), "payload is released");
                assert!(callback_weak.upgrade().is_none(), "callback is released");
                assert!(registry.entries.lock().unwrap().cases.is_empty());
            }
        })
        .expect("spawn small-stack case rejection test")
        .join()
        .expect("deep case rejection must not overflow");
}

#[test]
fn registered_cases_follow_claim_owner_cleanup_and_independent_lifetimes() {
    let registry = CaseRegistry::default();
    let domain = MemoryDomain::new();
    let first = registry.create_in_domain(1, &domain, case).expect("first");
    let second = registry.create_in_domain(2, &domain, case).expect("second");
    assert_eq!(domain.root_count(), 2);
    let claimed = registry.claim(1, &[first]).expect("claim");
    registry.close_owner(1);
    assert_eq!(domain.root_count(), 2, "claimed case stays rooted");
    let before = domain.maintenance_generation();
    registry.close_owner(2);
    assert!(!registry.close(2, second).expect("already closed"));
    assert_eq!(domain.root_count(), 1);
    assert!(domain.maintenance_generation() > before);
    let before = domain.maintenance_generation();
    drop(claimed);
    assert_eq!(domain.root_count(), 0);
    assert!(domain.maintenance_generation() > before);
}
