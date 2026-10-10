//! Generic parameter identities and built-in constraints.
//! See `docs/pascal/language/types/generics.md`.

use super::Ty;

/// Built-in capabilities required by a generic parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeConstraint {
    /// Supports equality and ordering comparisons.
    Comparable,
    /// Supports arithmetic operations.
    Numeric,
    /// Can be converted to a string representation.
    Printable,
}

impl TypeConstraint {
    /// Resolve a constraint name without case sensitivity.
    pub fn from_name(name: &str) -> Option<Self> {
        if name.eq_ignore_ascii_case("comparable") {
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
            Self::Comparable => "Comparable",
            Self::Numeric => "Numeric",
            Self::Printable => "Printable",
        }
    }

    /// Check a concrete type or the guarantees of an enclosing parameter.
    pub fn satisfied_by(self, ty: &Ty) -> bool {
        if let Ty::GenericParam(_, constraint) = ty {
            return constraint.is_some_and(|actual| {
                actual == self
                    || (actual == Self::Numeric
                        && matches!(self, Self::Comparable | Self::Printable))
            });
        }
        match self {
            // Distinct scalar types inherit comparisons, not arithmetic or printing.
            // See docs/pascal/language/types/distinct-types.md.
            Self::Comparable => matches!(
                ty,
                Ty::Integer | Ty::Real | Ty::Boolean | Ty::String | Ty::Distinct(_)
            ),
            Self::Numeric => matches!(ty, Ty::Integer | Ty::Real),
            Self::Printable => !matches!(ty, Ty::Function(_) | Ty::Procedure(_) | Ty::Distinct(_)),
        }
    }
}

/// A resolved parameter identity and its optional constraint.
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParamDef {
    /// Case-preserving identity, qualified by the declaring routine for record methods.
    pub name: String,
    /// Capabilities guaranteed for every permitted argument.
    pub constraint: Option<TypeConstraint>,
}

impl GenericParamDef {
    /// Source parameter name for diagnostics, excluding its declaring routine.
    pub fn display_name(&self) -> &str {
        parameter_name(&self.name)
    }
}

/// Source spelling of a parameter with a possibly qualified semantic identity.
pub(crate) fn parameter_name(identity: &str) -> &str {
    identity.rsplit('.').next().unwrap_or(identity)
}
