//! Runtime-only code provenance for validated first-class function targets.
//!
//! Documentation: `docs/pascal/language/functions/closures.md`.

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use super::VerifiedExecutable;
use crate::FunctionId;

/// Opaque identity of one independently verified executable instance.
///
/// Only executable verification and independent image cloning mint identities.
/// Holding this token does not retain the executable or grant cell-domain access.
#[derive(Debug, Clone)]
pub struct ExecutableIdentity(Arc<()>);

impl ExecutableIdentity {
    pub(super) fn new() -> Self {
        Self(Arc::new(()))
    }
}

impl PartialEq for ExecutableIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ExecutableIdentity {}

impl Hash for ExecutableIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(Arc::as_ptr(&self.0), state);
    }
}

/// A function index validated against one executable's function table.
///
/// Create targets with [`VerifiedExecutable::function_target`]. Fields and
/// constructors are private so safe host code cannot attach an arbitrary index
/// or diagnostic name to another executable's identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionTarget {
    identity: ExecutableIdentity,
    function: FunctionId,
    name: Arc<str>,
}

impl FunctionTarget {
    pub(super) fn new(identity: ExecutableIdentity, function: FunctionId, name: Arc<str>) -> Self {
        Self {
            identity,
            function,
            name,
        }
    }

    /// Return the function index validated within this target's executable.
    #[must_use]
    #[inline]
    pub const fn function(&self) -> FunctionId {
        self.function
    }

    /// Borrow the canonical function name prepared by executable verification.
    #[must_use]
    #[inline]
    pub fn name(&self) -> &Arc<str> {
        &self.name
    }

    /// Borrow the code identity without granting access to its executable.
    #[must_use]
    #[inline]
    pub fn identity(&self) -> &ExecutableIdentity {
        &self.identity
    }

    /// Test whether this target belongs to the exact verified code instance.
    #[must_use]
    pub fn belongs_to(&self, executable: &VerifiedExecutable) -> bool {
        &self.identity == executable.identity()
    }

    /// Test code identity independently of the two function indices.
    #[must_use]
    pub fn same_executable(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}
