//! Type compatibility and scalar capabilities.
//! See `docs/pascal/language/types/subranges.md`.
use super::{ParamTy, Ty, TypeConstraint};

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
        matches!((self, other), (Ty::Integer, Ty::Subrange(_)))
            || self.compatible_with_mode(other, false)
    }

    fn compatible_with_mode(&self, other: &Ty, generic_wildcard: bool) -> bool {
        if self.is_error() || other.is_error() {
            return true;
        }
        match (self, other) {
            (Ty::GenericParam(left, _), Ty::GenericParam(right, _)) => {
                left.eq_ignore_ascii_case(right)
            }
            (Ty::GenericParam(..), _) | (_, Ty::GenericParam(..)) => generic_wildcard,
            // Named type matches the concrete type with the same name (recursive enums).
            (Ty::Named(n), Ty::Enum(e)) | (Ty::Enum(e), Ty::Named(n)) => {
                n.eq_ignore_ascii_case(&e.name)
            }
            (Ty::Named(a), Ty::Named(b)) => a.eq_ignore_ascii_case(b),
            // Array with Error element type is compatible with any array
            (Ty::Array(a), Ty::Array(b)) => a.compatible_with_mode(b, generic_wildcard),
            (Ty::Channel(a), Ty::Channel(b)) => a.compatible_with_mode(b, generic_wildcard),
            // Named type matches the concrete record with the same name (recursive records).
            (Ty::Named(n), Ty::Record(r)) | (Ty::Record(r), Ty::Named(n)) => {
                n.eq_ignore_ascii_case(&r.name)
            }
            // Records are nominal.
            // Documentation: docs/pascal/language/types/records.md
            (Ty::Record(a), Ty::Record(b)) => {
                a.name.eq_ignore_ascii_case(&b.name)
                    && a.type_args.len() == b.type_args.len()
                    && a.type_args
                        .iter()
                        .zip(&b.type_args)
                        .all(|(left, right)| left.compatible_with_mode(right, generic_wildcard))
                    && match (&a.owner_unit, &b.owner_unit) {
                        (Some(a_owner), Some(b_owner)) => a_owner.eq_ignore_ascii_case(b_owner),
                        (None, None) => true,
                        _ => false,
                    }
            }
            // Distinct types are nominal and never mix with their underlying type.
            // Documentation: docs/pascal/language/types/distinct-types.md
            (Ty::Distinct(a), Ty::Distinct(b)) => a.same_declaration(b),
            // Enum applications preserve their nominal identity and arguments.
            // Documentation: docs/pascal/language/types/generics.md
            (Ty::Enum(a), Ty::Enum(b)) => {
                a.name.eq_ignore_ascii_case(&b.name)
                    && a.type_args.len() == b.type_args.len()
                    && a.type_args
                        .iter()
                        .zip(&b.type_args)
                        .all(|(left, right)| left.compatible_with_mode(right, generic_wildcard))
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
                if a.variadic != b.variadic || a.params.len() != b.params.len() {
                    return false;
                }
                a.return_type
                    .compatible_with_mode(&b.return_type, generic_wildcard)
                    && Self::params_compatible_with_mode(&a.params, &b.params, generic_wildcard)
            }
            (Ty::Procedure(a), Ty::Procedure(b)) => {
                if a.variadic != b.variadic || a.params.len() != b.params.len() {
                    return false;
                }
                Self::params_compatible_with_mode(&a.params, &b.params, generic_wildcard)
            }
            _ => self == other,
        }
    }

    /// True for numeric types (integer, real), or a generic param with Numeric constraint.
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Ty::Integer | Ty::Real | Ty::GenericParam(_, Some(TypeConstraint::Numeric))
        )
    }

    /// True for types that satisfy the Comparable constraint, including generic
    /// params with Comparable (or Numeric, since Numeric ⊂ Comparable).
    pub fn is_comparable(&self) -> bool {
        matches!(
            self,
            Ty::Integer
                | Ty::Subrange(_)
                | Ty::Real
                | Ty::Boolean
                | Ty::String
                | Ty::GenericParam(
                    _,
                    Some(TypeConstraint::Comparable | TypeConstraint::Numeric)
                )
        )
    }

    /// True for ordinal types (integer, boolean, simple enum without data).
    pub fn is_ordinal(&self) -> bool {
        matches!(self, Ty::Integer | Ty::Subrange(_) | Ty::Boolean)
            || matches!(self, Ty::Enum(e) if !e.has_data())
    }

    /// Parameter modes are part of callable compatibility: a read-only and a
    /// `var` parameter never substitute for each other.
    ///
    /// **Documentation:** `docs/pascal/language/functions/function-types.md`
    fn params_compatible_with_mode(
        params: &[ParamTy],
        other_params: &[ParamTy],
        generic_wildcard: bool,
    ) -> bool {
        params.iter().zip(other_params).all(|(param, other)| {
            param.mode == other.mode && param.ty.compatible_with_mode(&other.ty, generic_wildcard)
        })
    }
}
