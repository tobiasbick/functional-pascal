//! Sealed values crossing a host/session boundary.
//!
//! Documentation: `docs/pascal/program-structure/compiled-programs.md`.

mod admission;
mod mutation;
mod selection;

#[cfg(test)]
mod tests;

use std::fmt;
use std::sync::Arc;

use crate::{
    ExecutableIdentity, MemoryDomain, OwnershipError, RootKind, RootValues, Value, release_values,
};

pub use selection::{HostChild, HostEnumInfo};

/// An argument supplied by the embedding host.
///
/// Raw data must contain no local captures or session resources. Values exported by a session
/// may return to that session intact. Parameter types remain the host caller's responsibility.
#[derive(Debug)]
pub enum HostArgument {
    /// Independently constructed, transferable data.
    Data(Value),
    /// A sealed value previously returned by a VM or callback session.
    Export(HostValue),
}

impl From<Value> for HostArgument {
    fn from(value: Value) -> Self {
        Self::Data(value)
    }
}

impl From<HostValue> for HostArgument {
    fn from(value: HostValue) -> Self {
        Self::Export(value)
    }
}

impl Drop for HostArgument {
    fn drop(&mut self) {
        if let Self::Data(value) = self {
            release_values(vec![std::mem::replace(value, Value::Unit)]);
        }
    }
}

/// A returned value whose local captures and resource provenance remain sealed.
///
/// Cloning retains the same value and session origin. Use [`Self::try_to_data`] to extract a
/// wholly transferable graph, or [`Self::child`] to select another sealed value.
pub struct HostValue(Arc<ExportedValue>);

struct ExportedValue {
    value: Value,
    origin: Arc<Origin>,
    _roots: RootValues,
}

// The identity is the allocation, not a numeric session or task ID. Retaining an export keeps
// only its memory domain alive; it does not keep hosted registries, frames or an executable alive.
struct Origin {
    domain: MemoryDomain,
    executable: Option<ExecutableIdentity>,
}

impl Clone for HostValue {
    fn clone(&self) -> Self {
        let _access = self.0.origin.domain.access();
        Self::new(self.0.value.clone(), Arc::clone(&self.0.origin))
    }
}

impl PartialEq for HostValue {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.0.origin, &other.0.origin) {
            let _access = self.0.origin.domain.access();
            self.0.value == other.0.value
        } else {
            match (self.try_to_data(), other.try_to_data()) {
                (Ok(left), Ok(right)) => left == right,
                _ => false,
            }
        }
    }
}

impl Drop for ExportedValue {
    fn drop(&mut self) {
        release_values(vec![std::mem::replace(&mut self.value, Value::Unit)]);
    }
}

impl fmt::Debug for HostValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HostValue")
            .field("type_name", &self.type_name())
            .finish_non_exhaustive()
    }
}

/// A sealed value cannot be exposed as independent raw data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostValueError {
    /// The graph contains a local capture, reference, or session resource.
    SessionBound,
    /// The graph could not be read consistently.
    Ownership(OwnershipError),
    /// The selected value is not mutable cell storage.
    NotCell,
}

impl fmt::Display for HostValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SessionBound => formatter.write_str("value requires its originating session"),
            Self::Ownership(error) => error.fmt(formatter),
            Self::NotCell => formatter.write_str("value is not mutable cell storage"),
        }
    }
}

impl std::error::Error for HostValueError {}

impl HostValue {
    fn new(value: Value, origin: Arc<Origin>) -> Self {
        let roots = origin
            .domain
            .root_values(RootKind::Host, vec![value.clone()]);
        Self(Arc::new(ExportedValue {
            value,
            origin,
            _roots: roots,
        }))
    }

    /// Return the value's runtime category without exposing its payload.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        let _access = self.0.origin.domain.access();
        self.0.value.type_name()
    }

    /// Extract a graph containing neither local state nor session resources.
    ///
    /// A transferable callable retains its executable identity and can only be invoked against
    /// that same shared executable image.
    ///
    /// # Errors
    /// Returns an error if the graph is session-bound or cannot be inspected consistently.
    pub fn try_to_data(&self) -> Result<Value, HostValueError> {
        let _access = self.0.origin.domain.access();
        let (graph, resources) = admission::inspect(&self.0.value)?;
        if !graph.is_transferable() || resources {
            return Err(HostValueError::SessionBound);
        }
        Ok(self.0.value.clone())
    }

    #[cfg(test)]
    pub(in crate::execution::vm) fn runtime_value_for_test(&self) -> Value {
        self.0.value.clone()
    }
}

/// Export/import authority owned by one public VM or callback session.
pub(super) struct HostValues {
    origin: Arc<Origin>,
    _globals: Option<crate::RegisteredRoots>,
}

impl HostValues {
    #[cfg(test)]
    pub(super) fn new(domain: MemoryDomain) -> Self {
        Self {
            origin: Arc::new(Origin {
                domain,
                executable: None,
            }),
            _globals: None,
        }
    }

    pub(super) fn for_executable(
        domain: MemoryDomain,
        executable: &crate::VerifiedExecutable,
        globals: Arc<std::sync::RwLock<Vec<Option<Value>>>>,
    ) -> Self {
        let root = domain.register_roots(
            RootKind::Globals,
            Arc::new(super::domain::roots::GlobalRoots(globals)),
        );
        Self {
            origin: Arc::new(Origin {
                domain,
                executable: Some(executable.identity().clone()),
            }),
            _globals: Some(root),
        }
    }

    pub(in crate::execution::vm) fn retire_globals(
        &mut self,
        globals: &Arc<std::sync::RwLock<Vec<Option<Value>>>>,
    ) {
        let _access = self.origin.domain.access();
        let removed = std::mem::take(&mut *globals.write().unwrap_or_else(|e| e.into_inner()));
        release_values(removed.into_iter().flatten().collect());
        self._globals.take();
    }

    pub(in crate::execution::vm) fn export_value(&self, value: Value) -> HostValue {
        HostValue::new(value, Arc::clone(&self.origin))
    }

    pub(super) fn export(&self, execution: super::RuntimeExecution) -> super::Execution {
        super::Execution {
            value: self.export_value(execution.value),
            instruction_count: execution.instruction_count,
        }
    }
}
