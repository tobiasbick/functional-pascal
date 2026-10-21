//! VM domain construction installs explicit collection, without automatic scheduling.
#[cfg(test)]
mod tests;

use fpas_bytecode::{
    CollectionLimits, DomainMaintenance, DomainSession, MaintenancePolicy, MemoryDomain,
    OwnershipError, Value,
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
fn install(domain: &MemoryDomain) {
    domain.set_maintenance(Arc::new(LocalCollector), MaintenancePolicy::Explicit);
}
pub(in crate::vm) fn new_domain() -> MemoryDomain {
    let domain = MemoryDomain::new();
    install(&domain);
    domain
}
pub(in crate::vm) fn new_session_domain() -> (MemoryDomain, DomainSession) {
    let (domain, session) = MemoryDomain::new_session();
    install(&domain);
    (domain, session)
}
