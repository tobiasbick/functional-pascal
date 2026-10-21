//! Preflight all payload locks and release buffers before the first destructive write.
use super::super::{CellSnapshot, MemoryDomain, OwnedCell, OwnershipError, Value};
use std::sync::Arc;

/// A panic after commit starts may never reopen a partially cleared domain.
pub(super) struct CommitGuard<'a> {
    domain: &'a MemoryDomain,
    armed: bool,
    pub(super) cleared: usize,
}
impl CommitGuard<'_> {
    pub(super) fn finish(mut self) {
        self.armed = false;
    }
}
impl Drop for CommitGuard<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.domain.poison_collection();
            super::increment(&self.domain.0.collection.aborted, 1);
        }
    }
}
pub(super) fn clear<'a>(
    domain: &'a MemoryDomain,
    cells: &[OwnedCell],
) -> Result<CommitGuard<'a>, OwnershipError> {
    let mut locks = Vec::new();
    let mut removed = Vec::new();
    locks
        .try_reserve(cells.len())
        .map_err(|_| OwnershipError::ScratchUnavailable)?;
    removed
        .try_reserve(cells.len())
        .map_err(|_| OwnershipError::ScratchUnavailable)?;
    let empty = Arc::new(Value::Unit);
    // No mutation occurs until every required lock and commit buffer is available.
    for cell in cells {
        locks.push(cell.lock(true)?);
    }
    let cleared = locks
        .iter()
        .filter(|slot| !matches!(slot.as_ref(), Value::Unit))
        .count();
    let commit = CommitGuard {
        domain,
        armed: true,
        cleared,
    };
    for slot in &mut locks {
        removed.push(CellSnapshot::from_arc(std::mem::replace(
            &mut **slot,
            Arc::clone(&empty),
        )));
    }
    drop(locks);
    // Existing snapshot teardown drains deep values iteratively, outside cell locks.
    drop(removed);
    super::increment(&domain.0.collection.cleared, cleared);
    Ok(commit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DomainMaintenance, MaintenancePolicy};
    struct Interrupt;
    impl DomainMaintenance for Interrupt {
        fn maintain(&self, domain: &MemoryDomain, _: &[Value]) -> Result<(), OwnershipError> {
            let _commit = CommitGuard {
                domain,
                armed: true,
                cleared: 0,
            };
            panic!("interrupted committed teardown");
        }
    }
    #[test]
    fn interrupted_commit_permanently_rejects_execution_and_external_access() {
        let domain = MemoryDomain::new();
        let owner = domain.execution_roots().unwrap();
        owner.publish(vec![]);
        domain.set_maintenance(Arc::new(Interrupt), MaintenancePolicy::Explicit);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || domain.request_explicit_maintenance()
            ))
            .is_err()
        );
        assert_eq!(domain.phase(), crate::DomainPhase::Poisoned);
        assert!(matches!(owner.enter(), Err(OwnershipError::Poisoned)));
        assert!(matches!(domain.try_access(), Err(OwnershipError::Poisoned)));
        assert_eq!(domain.collection_stats().aborted_passes, 1);
    }

    #[test]
    fn published_wait_cannot_resume_after_another_thread_interrupts_commit() {
        let domain = MemoryDomain::new();
        let owner = domain.execution_roots().unwrap();
        let outer = owner.enter().unwrap();
        let inner = owner.enter().unwrap();
        owner.publish(vec![]);
        let wait = domain.publish_wait().expect("published nested execution");
        domain.set_maintenance(Arc::new(Interrupt), MaintenancePolicy::Explicit);
        std::thread::scope(|scope| {
            assert!(
                scope
                    .spawn(|| domain.request_explicit_maintenance())
                    .join()
                    .is_err()
            );
        });
        assert_eq!(domain.phase(), crate::DomainPhase::Poisoned);
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(wait))).is_err());
        // Both parked tokens must still release without underflow or retaining the gate.
        drop(inner);
        drop(outer);
        assert!(matches!(owner.enter(), Err(OwnershipError::Poisoned)));
        assert!(matches!(domain.try_access(), Err(OwnershipError::Poisoned)));
        assert_eq!(domain.collection_stats().aborted_passes, 1);
    }
}
