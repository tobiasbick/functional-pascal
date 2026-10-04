//! Declaration identities of generic parameters in compiled-unit interfaces.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use super::TypeConstraint;

/// Identity of one generic parameter, independent of its case-insensitive spelling.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct GenericParameterId {
    /// Canonical declaring unit, or `None` for program declarations.
    pub unit: Option<String>,
    /// Program source identifier; unit declarations use zero for independent builds.
    pub source_id: u32,
    /// Byte offset of the declared parameter within its source.
    pub offset: u64,
}

/// One generic parameter in a callable or nominal type declaration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GenericParameter {
    /// Source spelling of the parameter name.
    pub name: String,
    /// Optional built-in constraint.
    pub constraint: Option<TypeConstraint>,
    /// Identity retained by references and concrete substitution.
    pub identity: GenericParameterId,
}

impl GenericParameter {
    pub(super) fn canonicalize(&mut self) {
        self.identity.unit = self
            .identity
            .unit
            .as_ref()
            .map(|name| name.to_ascii_lowercase());
    }
}
