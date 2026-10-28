//! VM domains collect automatically with one effective inherited schedule.
#[cfg(test)]
mod tests;

use crate::{
    CollectionLimits, CollectionSchedule, DomainMaintenance, DomainSession, MaintenancePolicy,
    MemoryDomain, OwnershipError, Value,
};
use std::sync::Arc;

/// Stateless: retaining a domain must not retain a worker, session or callback registry.
struct LocalCollector;
impl DomainMaintenance for LocalCollector {
    fn maintain(&self, domain: &MemoryDomain, roots: &[Value]) -> Result<(), OwnershipError> {
        domain
            .reclaim_unreachable(roots, CollectionLimits::default())
            .map(|_| ())
    }
}
pub(in crate::execution::vm) fn configure(domain: &MemoryDomain, schedule: CollectionSchedule) {
    domain.set_maintenance(
        Arc::new(LocalCollector),
        MaintenancePolicy::Scheduled(schedule),
    );
}
fn scheduled_domain(schedule: CollectionSchedule) -> MemoryDomain {
    MemoryDomain::with_maintenance(
        Arc::new(LocalCollector),
        MaintenancePolicy::Scheduled(schedule),
    )
}
pub(in crate::execution::vm) fn new_domain() -> MemoryDomain {
    scheduled_domain(CollectionSchedule::default())
}
pub(in crate::execution::vm) fn related_domain(parent: &MemoryDomain) -> MemoryDomain {
    scheduled_domain(parent.collection_schedule().unwrap_or_default())
}
pub(in crate::execution::vm) fn new_session_domain() -> (MemoryDomain, DomainSession) {
    let (domain, session) = MemoryDomain::new_session();
    configure(&domain, CollectionSchedule::default());
    (domain, session)
}
