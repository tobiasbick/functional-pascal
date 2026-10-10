pub use fpas_parser::ParamMode;
use std::sync::Arc;

/// Enum identities, generic arguments, and resolved payloads.
mod enums;
/// Generic parameter identities and built-in constraints.
mod generic_parameters;
/// Partial type evidence collected from generic arguments.
mod inference;
/// Record identities, generic arguments, and resolved members.
mod records;
/// Substitution shared by generic routines and record instantiation.
mod substitution;
pub use enums::{EnumTy, EnumVariantTy};
pub(crate) use generic_parameters::parameter_name;
pub use generic_parameters::{GenericParamDef, TypeConstraint};
pub(crate) use inference::{has_inference_holes, merge_inferred_types, needs_expected_type};
pub use records::{MethodKind, RecordTy};
pub(crate) use substitution::substitute_type_parameters;

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
    /// Distinct domain type with its own identity over a scalar underlying type.
    ///
    /// **Documentation:** `docs/pascal/language/types/distinct-types.md`
    Distinct(Arc<DistinctTy>),
    /// Function signature.
    Function(FunctionTy),
    /// Procedure signature.
    Procedure(ProcedureTy),
    /// A named type not yet resolved or unknown.
    Named(String),
    /// `Result of (T, E)`.
    Result(Box<Ty>, Box<Ty>),
    /// `Option of T`.
    Option(Box<Ty>),
    /// A generic type parameter (e.g. `T` in `function Identity<T>`),
    /// optionally carrying its constraint for operator checking inside generic bodies.
    GenericParam(String, Option<TypeConstraint>),
    /// `dict of K to V` — key-value collection.
    ///
    /// **Documentation:** `docs/pascal/language/types/dictionaries.md`
    Dict(Box<Ty>, Box<Ty>),
    /// `task` — handle to a spawned concurrent task (return type erased at runtime).
    ///
    /// **Documentation:** `docs/pascal/language/concurrency/README.md`
    Task(Box<Ty>),
    /// Placeholder for errors — compatible with anything to avoid cascading.
    Error,
}

/// Nominal identity and underlying scalar type of a distinct domain type.
///
/// **Documentation:** `docs/pascal/language/types/distinct-types.md`
#[derive(Debug, Clone, PartialEq)]
pub struct DistinctTy {
    /// Case-preserving declared type name.
    pub name: String,
    /// Exact source unit that declared the type, or `None` for program-local types.
    pub owner_unit: Option<String>,
    /// Underlying `integer`, `real`, `string`, or `boolean` type.
    pub underlying: Ty,
}

impl DistinctTy {
    /// Whether both descriptors name the same declaration.
    pub fn same_declaration(&self, other: &DistinctTy) -> bool {
        self.name.eq_ignore_ascii_case(&other.name)
            && match (&self.owner_unit, &other.owner_unit) {
                (Some(left), Some(right)) => left.eq_ignore_ascii_case(right),
                (None, None) => true,
                _ => false,
            }
    }
}

/// Resolved function signature used by semantic analysis and editor tooling.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionTy {
    /// Generic parameters declared by the function.
    pub type_params: Vec<GenericParamDef>,
    /// Function parameters in source order.
    pub params: Vec<ParamTy>,
    /// Resolved function return type.
    pub return_type: Box<Ty>,
    /// Accept any number of arguments beyond the declared params (e.g. `Std.Str.Format`).
    pub variadic: bool,
}

/// Resolved procedure signature used by semantic analysis and editor tooling.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureTy {
    /// Generic parameters declared by the procedure.
    pub type_params: Vec<GenericParamDef>,
    /// Procedure parameters in source order.
    pub params: Vec<ParamTy>,
    /// Accept any number of arguments at the call site (e.g. `Std.Console.WriteLn`).
    pub variadic: bool,
}

/// One resolved callable parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct ParamTy {
    /// Case-preserving parameter name.
    pub name: String,
    /// Resolved parameter type.
    pub ty: Ty,
    /// Whether the parameter is read-only or a `var` parameter.
    ///
    /// **Documentation:** `docs/pascal/language/functions/var-parameters.md`
    pub mode: ParamMode,
}

impl ParamTy {
    /// Creates a read-only value parameter.
    #[must_use]
    pub fn value(name: impl Into<String>, ty: Ty) -> Self {
        Self {
            name: name.into(),
            ty,
            mode: ParamMode::Value,
        }
    }

    /// Returns whether the parameter changes the caller's variable.
    #[must_use]
    pub fn is_var(&self) -> bool {
        self.mode == ParamMode::Var
    }
}

impl std::fmt::Display for ParamTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_var() {
            write!(f, "var ")?;
        }
        write!(f, "{}: {}", self.name, self.ty)
    }
}

impl std::fmt::Display for Ty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ty::Integer => write!(f, "integer"),
            Ty::Real => write!(f, "real"),
            Ty::Boolean => write!(f, "boolean"),
            Ty::String => write!(f, "string"),
            Ty::Unit => write!(f, "unit"),
            Ty::Array(inner) => write!(f, "array of {inner}"),
            Ty::Channel(inner) => write!(f, "channel of {inner}"),
            Ty::Record(r) => {
                write!(f, "{}", r.name)?;
                if !r.type_args.is_empty() {
                    write!(f, " of ")?;
                    if r.type_args.len() > 1 {
                        write!(f, "(")?;
                    }
                    for (index, argument) in r.type_args.iter().enumerate() {
                        if index > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{argument}")?;
                    }
                    if r.type_args.len() > 1 {
                        write!(f, ")")?;
                    }
                }
                Ok(())
            }
            Ty::Enum(e) => {
                write!(f, "{}", e.name)?;
                if !e.type_args.is_empty() {
                    write!(f, " of ")?;
                    if e.type_args.len() > 1 {
                        write!(f, "(")?;
                    }
                    for (index, argument) in e.type_args.iter().enumerate() {
                        if index > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{argument}")?;
                    }
                    if e.type_args.len() > 1 {
                        write!(f, ")")?;
                    }
                }
                Ok(())
            }
            Ty::Distinct(d) => write!(f, "{}", d.name),
            Ty::Function(ft) => {
                write!(f, "function(")?;
                for (i, p) in ft.params.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{p}")?;
                }
                write!(f, "): {}", ft.return_type)
            }
            Ty::Procedure(pt) => {
                write!(f, "procedure(")?;
                for (i, p) in pt.params.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{p}")?;
                }
                write!(f, ")")
            }
            Ty::Named(n) => write!(f, "{n}"),
            Ty::Result(ok, err) => write!(f, "Result of ({ok}, {err})"),
            Ty::Option(inner) => write!(f, "Option of {inner}"),
            Ty::GenericParam(name, _) => write!(f, "{}", parameter_name(name)),
            Ty::Dict(k, v) => write!(f, "dict of {k} to {v}"),
            Ty::Task(inner) => write!(f, "task of {inner}"),
            Ty::Error => write!(f, "<error>"),
        }
    }
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
        matches!(self, Ty::Integer | Ty::Boolean) || matches!(self, Ty::Enum(e) if !e.has_data())
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

#[cfg(test)]
mod tests;
