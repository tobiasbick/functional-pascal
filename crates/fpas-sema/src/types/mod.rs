mod callables;
mod components;
mod constraints;
mod data;
mod display;
mod generics;
mod inference;
mod substitution;

/// Resolved callable signatures and parameter types.
pub use callables::{FunctionTy, ParamTy, ProcedureTy};

/// Built-in generic constraints and their capability rules.
pub use constraints::TypeConstraint;

/// Nominal value descriptors and record member metadata.
pub use data::{EnumTy, EnumVariantTy, RecordTy};

pub(crate) use generics::TypeArguments;
/// Generic declaration identities and capability requirements.
pub use generics::{GenericParamDef, GenericParameterId};

use std::sync::Arc;

/// Resolved type representation used during semantic analysis.
///
/// **Documentation:** `docs/pascal/language/types/generics.md`
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    /// Signed integer value.
    Integer,
    /// Double-precision real value.
    Real,
    /// Boolean value.
    Boolean,
    /// UTF-8 string value.
    String,
    /// Procedure / void result (e.g. `Std.Arrays.Push`).
    Unit,
    /// An array whose elements have the enclosed type.
    Array(Box<Ty>),
    /// A bounded FIFO channel carrying the enclosed type.
    ///
    /// **Documentation:** `docs/pascal/language/types/channels.md`
    Channel(Box<Ty>),
    /// Shared descriptor for a record type.
    Record(Arc<RecordTy>),
    /// Shared descriptor for an enum type.
    Enum(Arc<EnumTy>),
    /// Function signature.
    Function(FunctionTy),
    /// Procedure signature.
    Procedure(ProcedureTy),
    /// A named type not yet resolved or unknown.
    Named(String),
    /// A deferred application of a nominal generic type, including recursive references.
    Applied(String, Vec<Ty>),
    /// `Result of (T, E)`.
    Result(Box<Ty>, Box<Ty>),
    /// `Option of (T)`.
    Option(Box<Ty>),
    /// A generic type parameter (e.g. `T` in `function Identity of (T)`),
    /// optionally carrying its constraint for operator checking inside generic bodies.
    GenericParam(Arc<GenericParamDef>),
    /// `dict of (K, V)` — key-value collection.
    ///
    /// **Documentation:** `docs/pascal/language/types/dictionaries.md`
    Dict(Box<Ty>, Box<Ty>),
    /// Handle to a spawned concurrent task with its checked result type.
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/README.md`
    Task(Box<Ty>),
    /// Placeholder for errors — compatible with anything to avoid cascading.
    Error,
}

impl Ty {
    /// Returns true if this type is the error sentinel.
    pub fn is_error(&self) -> bool {
        matches!(self, Ty::Error)
    }

    /// Returns true if both types are compatible (same type or one is Error).
    pub fn compatible_with(&self, other: &Ty) -> bool {
        self.compatible_with_mode(other, true)
    }

    /// Returns true when ordinary assignment may use `other` as this type.
    pub(crate) fn assignment_compatible_with(&self, other: &Ty) -> bool {
        self.compatible_with_mode(other, false)
    }

    fn compatible_with_mode(&self, other: &Ty, generic_wildcard: bool) -> bool {
        if self.is_error() || other.is_error() {
            return true;
        }
        match (self, other) {
            (Ty::GenericParam(left), Ty::GenericParam(right)) => left.identity == right.identity,
            (Ty::GenericParam(..), _) | (_, Ty::GenericParam(..)) => generic_wildcard,
            // Named type matches the concrete type with the same name (recursive enums).
            (Ty::Named(n), Ty::Enum(e)) | (Ty::Enum(e), Ty::Named(n)) => {
                n.eq_ignore_ascii_case(&e.name)
            }
            (Ty::Named(a), Ty::Named(b)) => a.eq_ignore_ascii_case(b),
            (Ty::Applied(a, left), Ty::Applied(b, right)) => {
                a.eq_ignore_ascii_case(b)
                    && Self::arguments_compatible(left, right, generic_wildcard)
            }
            (Ty::Applied(name, arguments), Ty::Record(record))
            | (Ty::Record(record), Ty::Applied(name, arguments)) => {
                name.eq_ignore_ascii_case(&record.name)
                    && Self::arguments_compatible(arguments, &record.type_args, generic_wildcard)
            }
            (Ty::Applied(name, arguments), Ty::Enum(enumeration))
            | (Ty::Enum(enumeration), Ty::Applied(name, arguments)) => {
                name.eq_ignore_ascii_case(&enumeration.name)
                    && Self::arguments_compatible(
                        arguments,
                        &enumeration.type_args,
                        generic_wildcard,
                    )
            }
            // Array with Error element type is compatible with any array
            (Ty::Array(a), Ty::Array(b)) => a.compatible_with_mode(b, generic_wildcard),
            (Ty::Channel(a), Ty::Channel(b)) => {
                a.compatible_with_mode(b, generic_wildcard)
                    && b.compatible_with_mode(a, generic_wildcard)
            }
            // Named type matches the concrete record with the same name (recursive records).
            (Ty::Named(n), Ty::Record(r)) | (Ty::Record(r), Ty::Named(n)) => {
                n.eq_ignore_ascii_case(&r.name)
            }
            // Record identity includes the declaring unit and concrete type arguments.
            (Ty::Record(a), Ty::Record(b)) => {
                a.name.eq_ignore_ascii_case(&b.name)
                    && Self::arguments_compatible(&a.type_args, &b.type_args, generic_wildcard)
                    && match (&a.owner_unit, &b.owner_unit) {
                        (Some(a_owner), Some(b_owner)) => a_owner.eq_ignore_ascii_case(b_owner),
                        (None, None) => true,
                        _ => false,
                    }
            }
            (Ty::Enum(a), Ty::Enum(b)) => {
                a.name.eq_ignore_ascii_case(&b.name)
                    && Self::arguments_compatible(&a.type_args, &b.type_args, generic_wildcard)
            }
            (Ty::Unit, Ty::Unit) => true,
            // Result covariance
            (Ty::Result(ok1, err1), Ty::Result(ok2, err2)) => {
                ok1.compatible_with_mode(ok2, generic_wildcard)
                    && err1.compatible_with_mode(err2, generic_wildcard)
            }
            // Option covariance
            (Ty::Option(a), Ty::Option(b)) => a.compatible_with_mode(b, generic_wildcard),
            // Task covariance (inner type may be erased as Error)
            (Ty::Task(a), Ty::Task(b)) => a.compatible_with_mode(b, generic_wildcard),
            // Dict covariance
            (Ty::Dict(k1, v1), Ty::Dict(k2, v2)) => {
                k1.compatible_with_mode(k2, generic_wildcard)
                    && v1.compatible_with_mode(v2, generic_wildcard)
            }
            // Function and procedure structural compatibility: variadic flag, param count,
            // per-parameter mutability, and element-wise type compatibility. This allows
            // generic params inside function-typed parameters to unify with concrete types
            // at call sites (e.g., `function(X: T): R` vs `function(X: integer): string`
            // when T=integer, R=string).
            (Ty::Function(a), Ty::Function(b)) => {
                if a.pure && !b.pure {
                    return false;
                }
                if a.variadic != b.variadic || a.params.len() != b.params.len() {
                    return false;
                }
                a.return_type
                    .compatible_with_mode(&b.return_type, generic_wildcard)
                    && a.params.iter().zip(b.params.iter()).all(|(pa, pb)| {
                        pa.mutable == pb.mutable
                            && pb.ty.compatible_with_mode(&pa.ty, generic_wildcard)
                            && (!pa.mutable || pa.ty.compatible_with_mode(&pb.ty, generic_wildcard))
                    })
            }
            (Ty::Procedure(a), Ty::Procedure(b)) => {
                if a.variadic != b.variadic || a.params.len() != b.params.len() {
                    return false;
                }
                a.params.iter().zip(b.params.iter()).all(|(pa, pb)| {
                    pa.mutable == pb.mutable
                        && pb.ty.compatible_with_mode(&pa.ty, generic_wildcard)
                        && (!pa.mutable || pa.ty.compatible_with_mode(&pb.ty, generic_wildcard))
                })
            }
            _ => self == other,
        }
    }

    /// Check structural equality using the owning checker to resolve nominal references.
    pub(crate) fn supports_equality_with(&self, resolve: impl Fn(&Ty) -> Ty) -> bool {
        components::supports_equality(self, resolve)
    }

    /// True for numeric types (integer, real), or a generic param with Numeric constraint.
    pub fn is_numeric(&self) -> bool {
        matches!(self, Ty::Integer | Ty::Real)
            || matches!(self, Ty::GenericParam(parameter) if parameter.constraint == Some(TypeConstraint::Numeric))
    }

    /// True for types that satisfy the Comparable constraint, including generic
    /// params with Comparable (or Numeric, since Numeric ⊂ Comparable).
    pub fn is_comparable(&self) -> bool {
        matches!(self, Ty::Integer | Ty::Real | Ty::Boolean | Ty::String)
            || matches!(self, Ty::GenericParam(parameter) if matches!(parameter.constraint, Some(TypeConstraint::Comparable | TypeConstraint::Numeric)))
    }

    /// True for ordinal types (integer, boolean, simple enum without data).
    pub fn is_ordinal(&self) -> bool {
        matches!(self, Ty::Integer | Ty::Boolean) || matches!(self, Ty::Enum(e) if !e.has_data())
    }

    fn arguments_compatible(left: &[Ty], right: &[Ty], generic_wildcard: bool) -> bool {
        left.len() == right.len()
            && left.iter().zip(right).all(|(left, right)| {
                left.compatible_with_mode(right, generic_wildcard)
                    && right.compatible_with_mode(left, generic_wildcard)
            })
    }
}

#[cfg(test)]
mod tests;
