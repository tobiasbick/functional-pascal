pub use fpas_parser::ParamMode;
use std::sync::Arc;
mod array_pop;
mod category;
/// Type equality, assignment compatibility, and scalar capabilities.
mod compatibility;
mod equality;
pub use array_pop::{array_pop_result, array_pop_result_declaration};

/// Enum identities, generic arguments, and resolved payloads.
mod enums;
/// Generic parameter identities and built-in constraints.
mod generic_parameters;
mod identity;
/// Partial type evidence collected from generic arguments.
mod inference;
mod payload_constructors;
pub use identity::{error_schema, semantic_type_identity};
/// Record identities, generic arguments, and resolved members.
mod records;
/// Inclusive integer bounds and structural subrange identity.
mod subranges;
/// Substitution shared by generic routines and record instantiation.
mod substitution;
pub use enums::{EnumTy, EnumVariantTy};
pub(crate) use generic_parameters::parameter_name;
pub use generic_parameters::{GenericParamDef, TypeConstraint};
pub(crate) use inference::{has_inference_holes, merge_inferred_types, needs_expected_type};
pub use payload_constructors::PayloadConstructor;
pub use records::{MethodKind, RecordTy};
pub(crate) use records::{native_resource_fresh_result, native_resource_release};
pub use subranges::SubrangeTy;
pub(crate) use substitution::substitute_type_parameters;

/// Resolved type representation used during semantic analysis.
///
/// **Documentation:** `docs/pascal/language/types/generics.md`
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    /// Signed integer value.
    Integer,
    /// Integer restricted to inclusive compile-time bounds.
    ///
    /// **Documentation:** `docs/pascal/language/types/subranges.md`
    Subrange(SubrangeTy),
    /// Double-precision real value.
    Real,
    /// Boolean value.
    Boolean,
    /// UTF-8 string value.
    String,
    /// Native managed `Std.Bytes.ByteBuffer` value.
    ByteBuffer,
    /// Procedure / void result (e.g. array PushInPlace).
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
            Ty::Subrange(range) => write!(f, "{}", range.name),
            Ty::Real => write!(f, "real"),
            Ty::Boolean => write!(f, "boolean"),
            Ty::String => write!(f, "string"),
            Ty::ByteBuffer => write!(f, "Std.Bytes.ByteBuffer"),
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
            Ty::Task(inner) if matches!(inner.as_ref(), Ty::Unit) => write!(f, "task"),
            Ty::Task(inner) => write!(f, "task of {inner}"),
            Ty::Error => write!(f, "<error>"),
        }
    }
}

#[cfg(test)]
mod tests;
