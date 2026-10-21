//! VM domains collect automatically with one effective inherited schedule.
#[cfg(test)]
mod tests;

use fpas_bytecode::{
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
pub(in crate::vm) fn configure(domain: &MemoryDomain, schedule: CollectionSchedule) {
    domain.set_maintenance(
        Arc::new(LocalCollector),
        MaintenancePolicy::Scheduled(schedule),
    );
}
pub(in crate::vm) fn new_domain() -> MemoryDomain {
    let domain = MemoryDomain::new();
    configure(&domain, CollectionSchedule::default());
    domain
}
pub(in crate::vm) fn related_domain(parent: &MemoryDomain) -> MemoryDomain {
    let domain = MemoryDomain::new();
    configure(&domain, parent.collection_schedule().unwrap_or_default());
    domain
}
pub(in crate::vm) fn new_session_domain() -> (MemoryDomain, DomainSession) {
    let (domain, session) = MemoryDomain::new_session();
    configure(&domain, CollectionSchedule::default());
    (domain, session)
}
