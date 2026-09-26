//! Atomic case claims, resource limits, and task/VM ownership regressions.

use super::*;
use fpas_bytecode::FunctionId;

fn case() -> WaitCase {
    WaitCase {
        source: CaseSource::Timer(0),
        callback: SharedFunction::unbound(FunctionId::new(0), "callback", vec![]),
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
        assert!(registry.entries.lock().unwrap().is_empty());
    }
}

#[test]
fn close_claim_and_registry_teardown_release_callback_captures() {
    use std::sync::Arc;
    for path in 0..3 {
        let registry = CaseRegistry::default();
        let captured = Arc::new(Mutex::new(Value::Integer(42)));
        let weak = Arc::downgrade(&captured);
        let id = registry
            .create(0, || WaitCase {
                source: CaseSource::Timer(0),
                callback: SharedFunction::task_owned(
                    FunctionId::new(0),
                    "callback",
                    vec![Value::Cell(captured)],
                    0,
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
