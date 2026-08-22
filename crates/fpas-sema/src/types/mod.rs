use std::sync::Arc;

/// Built-in type constraints for generic parameters.
///
/// **Documentation:** `docs/pascal/language/types/generics.md` (Generics — Constraints)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeConstraint {
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

    /// Check whether a concrete type satisfies this constraint.
    pub fn satisfied_by(self, ty: &Ty) -> bool {
        match self {
            Self::Comparable => matches!(ty, Ty::Integer | Ty::Real | Ty::Boolean | Ty::String),
            Self::Numeric => matches!(ty, Ty::Integer | Ty::Real),
            Self::Printable => !matches!(ty, Ty::Function(_) | Ty::Procedure(_)),
        }
    }
}

/// A resolved generic type parameter with optional constraint.
///
/// **Documentation:** `docs/pascal/language/types/generics.md` (Generics — Constraints)
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParamDef {
    pub name: String,
    pub constraint: Option<TypeConstraint>,
}

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
    /// Procedure / void result (e.g. `Std.Array.Push`).
    Unit,
    /// An array whose elements have the enclosed type.
    Array(Box<Ty>),
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
    /// `Result of T, E`.
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

/// Resolved record shape, ownership, members, and callable metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordTy {
    /// Case-preserving qualified record name.
    pub name: String,
    /// Exact source unit that declared the record, or `None` for local and intrinsic records.
    pub owner_unit: Option<String>,
    /// Case-preserving names of members not declared `public`.
    pub private_members: Vec<String>,
    /// Declared record fields in source order.
    pub fields: Vec<(String, Ty)>,
    /// Instance methods (require implicit `Self`).
    pub methods: Vec<(String, MethodKind)>,
    /// Static functions called through the type name (no receiver).
    ///
    /// **Documentation:** `docs/pascal/language/types/record-methods.md`
    pub static_functions: Vec<(String, FunctionTy)>,
    /// Static procedures called through the type name (no receiver).
    ///
    /// **Documentation:** `docs/pascal/language/types/record-methods.md`
    pub static_procedures: Vec<(String, ProcedureTy)>,
    /// Computed properties backed by instance accessors.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-properties.md`
    pub properties: Vec<(String, PropertyTy)>,
    /// Event members backed by `Option of Handler` accessors.
    ///
    /// **Documentation:** `docs/pascal/language/types/record-events.md`
    pub events: Vec<(String, EventTy)>,
}

/// A computed record property and its resolved accessor names.
///
/// **Documentation:** `docs/pascal/language/types/record-properties.md`
#[derive(Debug, Clone, PartialEq)]
pub struct PropertyTy {
    /// Declared property type.
    pub ty: Ty,
    /// Qualified getter name (`Record.GetText`), when readable.
    pub getter: Option<String>,
    /// Qualified setter name (`Record.SetText`), when writable.
    pub setter: Option<String>,
}

/// A record event and its resolved `Option of Handler` accessors.
///
/// **Documentation:** `docs/pascal/language/types/record-events.md`
#[derive(Debug, Clone, PartialEq)]
pub struct EventTy {
    /// Declared handler callable type (function or procedure).
    pub handler_ty: Ty,
    /// Qualified getter name returning `Option of` the handler type.
    pub getter: String,
    /// Qualified setter name accepting `Option of` the handler type.
    pub setter: String,
    /// Declaring unit prefix of the record type name, or `None` for program-local types.
    pub owner_unit: Option<String>,
}

/// Whether a record method is a function (returns a value) or a procedure.
#[derive(Debug, Clone, PartialEq)]
pub enum MethodKind {
    Function(FunctionTy),
    Procedure(ProcedureTy),
}

/// **Documentation:** `docs/pascal/language/types/enums.md`
#[derive(Debug, Clone, PartialEq)]
pub struct EnumTy {
    /// Case-preserving qualified enum name.
    pub name: String,
    /// Declared variants in source order.
    pub variants: Vec<EnumVariantTy>,
}

/// A single variant in an enum type. Simple variants have an empty `fields` vec.
///
/// **Documentation:** `docs/pascal/language/types/enums.md`
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariantTy {
    pub name: String,
    pub fields: Vec<(String, Ty)>,
    /// Declared or implicit integer value for a simple enum member.
    pub backing_value: Option<i64>,
}

impl EnumTy {
    /// True when at least one variant carries associated data.
    pub fn has_data(&self) -> bool {
        self.variants.iter().any(|v| !v.fields.is_empty())
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
    /// Whether the parameter may be mutated by the callee.
    pub mutable: bool,
    /// Case-preserving parameter name.
    pub name: String,
    /// Resolved parameter type.
    pub ty: Ty,
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
            Ty::Record(r) => write!(f, "{}", r.name),
            Ty::Enum(e) => write!(f, "{}", e.name),
            Ty::Function(ft) => {
                write!(f, "function(")?;
                for (i, p) in ft.params.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}: {}", p.name, p.ty)?;
                }
                write!(f, "): {}", ft.return_type)
            }
            Ty::Procedure(pt) => {
                write!(f, "procedure(")?;
                for (i, p) in pt.params.iter().enumerate() {
                    if i > 0 {
                        write!(f, "; ")?;
                    }
                    write!(f, "{}: {}", p.name, p.ty)?;
                }
                write!(f, ")")
            }
            Ty::Named(n) => write!(f, "{n}"),
            Ty::Result(ok, err) => write!(f, "Result of {ok}, {err}"),
            Ty::Option(inner) => write!(f, "Option of {inner}"),
            Ty::GenericParam(name, _) => write!(f, "{name}"),
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
            // Named type matches the concrete record with the same name (recursive records).
            (Ty::Named(n), Ty::Record(r)) | (Ty::Record(r), Ty::Named(n)) => {
                n.eq_ignore_ascii_case(&r.name)
            }
            // Records: structural compatibility (ignore name)
            (Ty::Record(a), Ty::Record(b)) => {
                if !a.private_members.is_empty() || !b.private_members.is_empty() {
                    a.name.eq_ignore_ascii_case(&b.name)
                } else {
                    Self::record_fields_compatible_with_mode(&a.fields, &b.fields, generic_wildcard)
                }
            }
            // Enums: same name is sufficient (type-erased generics).
            (Ty::Enum(a), Ty::Enum(b)) => a.name.eq_ignore_ascii_case(&b.name),
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
                    && a.params.iter().zip(b.params.iter()).all(|(pa, pb)| {
                        pa.mutable == pb.mutable
                            && pa.ty.compatible_with_mode(&pb.ty, generic_wildcard)
                    })
            }
            (Ty::Procedure(a), Ty::Procedure(b)) => {
                if a.variadic != b.variadic || a.params.len() != b.params.len() {
                    return false;
                }
                a.params.iter().zip(b.params.iter()).all(|(pa, pb)| {
                    pa.mutable == pb.mutable && pa.ty.compatible_with_mode(&pb.ty, generic_wildcard)
                })
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

    /// Compare record fields under ordinary assignment rules.
    pub(crate) fn record_fields_assignment_compatible(
        fields: &[(String, Ty)],
        other_fields: &[(String, Ty)],
    ) -> bool {
        Self::record_fields_compatible_with_mode(fields, other_fields, false)
    }

    fn record_fields_compatible_with_mode(
        fields: &[(String, Ty)],
        other_fields: &[(String, Ty)],
        generic_wildcard: bool,
    ) -> bool {
        if fields.len() != other_fields.len() {
            return false;
        }

        fields.iter().all(|(name, ty)| {
            other_fields
                .iter()
                .find(|(other_name, _)| other_name.eq_ignore_ascii_case(name))
                .is_some_and(|(_, other_ty)| ty.compatible_with_mode(other_ty, generic_wildcard))
        }) && other_fields.iter().all(|(name, ty)| {
            fields
                .iter()
                .find(|(other_name, _)| other_name.eq_ignore_ascii_case(name))
                .is_some_and(|(_, other_ty)| ty.compatible_with_mode(other_ty, generic_wildcard))
        })
    }
}

#[cfg(test)]
mod tests;
