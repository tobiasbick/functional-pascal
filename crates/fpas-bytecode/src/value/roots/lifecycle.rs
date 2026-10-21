//! Session activity is distinct from invocation/frame ownership.
use super::super::MemoryDomain;

/// Observable domain protocol phase, independent of numeric scheduler identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainPhase {
    /// Session is active without an executing invocation.
    Idle,
    /// One thread owns execution access, possibly with nested callbacks.
    Executing,
    /// External graph access owns the gate.
    ExternalAccess,
    /// Execution owners are parked/stopped; root completeness is checked separately.
    Published,
    /// An exclusive maintenance pass owns the gate.
    Collecting,
    /// No execution or session owner remains; graph inspection/mutation is still valid.
    Retired,
    /// Interrupted committed teardown; no normal execution or graph access may resume.
    Poisoned,
}
/// Keeps a reusable callback/sandbox domain active between invocations.
pub struct DomainSession {
    domain: MemoryDomain,
}
impl MemoryDomain {
    /// Construct fresh session authority without admitting any execution yet.
    #[must_use]
    pub fn new_session() -> (Self, DomainSession) {
        let domain = Self::new();
        domain
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .sessions = 1;
        let session = DomainSession {
            domain: domain.clone(),
        };
        (domain, session)
    }

    /// Retain session activity. Closed domains cannot be revived.
    /// # Errors
    /// Rejects a retired domain.
    pub fn retain_session(&self) -> Result<DomainSession, super::super::OwnershipError> {
        let _access = self.access();
        let mut state = self
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if state.retired {
            return Err(super::super::OwnershipError::Retired);
        }
        state.sessions += 1;
        Ok(DomainSession {
            domain: self.clone(),
        })
    }
    /// Observe lifecycle without acquiring execution or traversing values.
    #[must_use]
    pub fn phase(&self) -> DomainPhase {
        let state = self
            .0
            .roots
            .access
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if state.poisoned {
            DomainPhase::Poisoned
        } else if state.collecting {
            DomainPhase::Collecting
        } else if state.execution != 0 {
            DomainPhase::Executing
        } else if state.external != 0 {
            DomainPhase::ExternalAccess
        } else if state.retired {
            DomainPhase::Retired
        } else if state.executions != 0 {
            DomainPhase::Published
        } else {
            DomainPhase::Idle
        }
    }
}
impl Drop for DomainSession {
    fn drop(&mut self) {
        {
            let mut state = self
                .domain
                .0
                .roots
                .access
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            state.sessions -= 1;
            if state.sessions == 0 && state.executions == 0 {
                state.retired = true;
            }
        }
        self.domain.request_maintenance();
    }
}
