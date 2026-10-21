//! Reclamation observations through real registered roots and exclusive maintenance.
mod empty;
mod failures;
mod graphs;
mod roots;
use super::*;
use crate::{DomainMaintenance, MaintenanceOutcome, MaintenancePolicy, RootKind};
use std::sync::Arc;

struct Collector(CollectionLimits);
impl DomainMaintenance for Collector {
    fn maintain(&self, domain: &MemoryDomain, roots: &[Value]) -> Result<(), OwnershipError> {
        domain.reclaim_unreachable(roots, self.0).map(|_| ())
    }
}
fn domain() -> MemoryDomain {
    let domain = MemoryDomain::new();
    domain.set_maintenance(
        Arc::new(Collector(CollectionLimits::default())),
        MaintenancePolicy::Explicit,
    );
    domain
}
fn collect(domain: &MemoryDomain) {
    assert!(matches!(
        domain.request_explicit_maintenance(),
        MaintenanceOutcome::Serviced(_)
    ));
}
fn cycle(domain: &MemoryDomain) -> OwnedCell {
    let cell = domain.new_cell(Value::Unit).unwrap();
    cell.replace(domain, Value::option_some(Value::Cell(cell.clone())))
        .unwrap();
    cell
}
#[test]
fn a_self_cycle_is_collected_only_after_its_last_registered_root_ends() {
    let domain = domain();
    let cell = cycle(&domain);
    let weak = cell.downgrade();
    let root = domain.root_values(RootKind::Host, vec![Value::Cell(cell)]);
    collect(&domain);
    assert!(weak.upgrade().is_some());
    drop(root);
    assert!(
        weak.upgrade().is_some(),
        "ordinary requests do not activate collection"
    );
    collect(&domain);
    assert!(weak.upgrade().is_none());
    assert_eq!(domain.collection_stats().cleared_cells, 1);
    assert_eq!(domain.collection_stats().released_cells, 1);
    collect(&domain);
    assert_eq!(domain.collection_stats().cleared_cells, 1);
}
