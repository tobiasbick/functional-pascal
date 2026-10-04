//! Built-in generic capabilities, including checks on forwarded type parameters.
//!
//! **Documentation:** `docs/pascal/language/types/generics.md`.

use super::Ty;

/// Built-in type constraints for generic parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeConstraint {
    /// Supports structural equality operators: `=` and `<>`.
    Equatable,
    /// Supports comparison operators: `=`, `<>`, `<`, `>`, `<=`, `>=`.
    Comparable,
    /// Supports arithmetic operators: `+`, `-`, `*`, `/`, `div`, `mod`.
    Numeric,
    /// Can be converted to a string representation.
    Printable,
}

impl TypeConstraint {
    /// Resolve a constraint name (case-insensitive) to a built-in constraint.
    pub fn from_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("equatable") {
            Some(Self::Equatable)
        } else if name.eq_ignore_ascii_case("comparable") {
            Some(Self::Comparable)
        } else if name.eq_ignore_ascii_case("numeric") {
            Some(Self::Numeric)
        } else if name.eq_ignore_ascii_case("printable") {
            Some(Self::Printable)
        } else {
            None
        }
    }

    /// Human-readable name for diagnostics.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::Equatable => "Equatable",
            Self::Comparable => "Comparable",
            Self::Numeric => "Numeric",
            Self::Printable => "Printable",
        }
    }

    /// Check a concrete type or a generic parameter's declared capability.
    ///
    /// An unconstrained parameter does not guarantee any constrained capability.
    pub fn satisfied_by(self, ty: &Ty) -> bool {
        self.satisfied_by_with(ty, |ty| ty.clone())
    }

    /// Check a capability while resolving recursive nominal type references.
    pub(crate) fn satisfied_by_with(self, ty: &Ty, resolve: impl Fn(&Ty) -> Ty) -> bool {
        if let Ty::GenericParam(_, constraint) = ty {
            return constraint.is_some_and(|actual| actual.implies(self));
        }
        match self {
            Self::Equatable => super::components::supports_equality(ty, resolve),
            Self::Comparable => matches!(ty, Ty::Integer | Ty::Real | Ty::Boolean | Ty::String),
            Self::Numeric => matches!(ty, Ty::Integer | Ty::Real),
            Self::Printable => !matches!(ty, Ty::Function(_) | Ty::Procedure(_)),
        }
    }

    /// Whether every type with this capability also provides the required capability.
    pub(super) fn implies(self, required: Self) -> bool {
        self == required
            || matches!(
                (self, required),
                (
                    Self::Numeric,
                    Self::Equatable | Self::Comparable | Self::Printable
                ) | (Self::Comparable, Self::Equatable | Self::Printable)
                    | (Self::Equatable, Self::Printable)
            )
    }
}
