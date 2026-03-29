/// Built-in type constraints for generic parameters.
///
/// **Documentation:** `docs/pascal/05-types.md` (Generics — Constraints)
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
        match name.to_ascii_lowercase().as_str() {
            "comparable" => Some(Self::Comparable),
            "numeric" => Some(Self::Numeric),
            "printable" => Some(Self::Printable),
            _ => None,
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
            Self::Comparable => matches!(
                ty,
                Ty::Integer | Ty::Real | Ty::Boolean | Ty::Char | Ty::String
            ),
            Self::Numeric => matches!(ty, Ty::Integer | Ty::Real),
            Self::Printable => !matches!(ty, Ty::Function(_) | Ty::Procedure(_)),
        }
    }
}

/// A resolved generic type parameter with optional constraint.
///
/// **Documentation:** `docs/pascal/05-types.md` (Generics — Constraints)
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParamDef {
    pub name: String,
    pub constraint: Option<TypeConstraint>,
}

impl GenericParamDef {
    /// Create an unconstrained parameter.
    pub fn unconstrained(name: String) -> Self {
        Self {
            name,
            constraint: None,
        }
    }
}

/// Resolved type representation used during semantic analysis.
///
/// **Documentation:** `docs/pascal/05-types.md`
#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Integer,
    Real,
    Boolean,
    Char,
    String,
    /// Procedure / void result (e.g. `Std.Array.Push`).
    Unit,
    Array(Box<Ty>),
    Record(RecordTy),
    Enum(EnumTy),
    Function(FunctionTy),
    Procedure(ProcedureTy),
    /// A named type not yet resolved or unknown.
    Named(std::string::String),
    /// `Result of T, E`.
    Result(Box<Ty>, Box<Ty>),
    /// `Option of T`.
    Option(Box<Ty>),
    /// A generic type parameter (e.g. `T` in `function Identity<T>`).
    GenericParam(std::string::String),
    /// `channel of T` — typed channel for concurrent communication.
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    Channel(Box<Ty>),
    /// `dict of K to V` — key-value collection.
    ///
    /// **Documentation:** `docs/future/advanced-types.md`
    Dict(Box<Ty>, Box<Ty>),
    /// `task` — handle to a spawned concurrent task (return type erased at runtime).
    ///
    /// **Documentation:** `docs/pascal/08-concurrency.md`
    Task(Box<Ty>),
    /// Placeholder for errors — compatible with anything to avoid cascading.
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordTy {
    pub name: std::string::String,
    /// Generic type parameters declared on this record.
    pub type_params: Vec<GenericParamDef>,
    pub fields: Vec<(std::string::String, Ty)>,
    pub methods: Vec<(std::string::String, MethodKind)>,
}

/// Whether a record method is a function (returns a value) or a procedure.
#[derive(Debug, Clone, PartialEq)]
pub enum MethodKind {
    Function(FunctionTy),
    Procedure(ProcedureTy),
}

/// **Documentation:** `docs/pascal/05-types.md`
#[derive(Debug, Clone, PartialEq)]
pub struct EnumTy {
    pub name: std::string::String,
    /// Generic type parameters declared on this enum.
    pub type_params: Vec<GenericParamDef>,
    pub variants: Vec<EnumVariantTy>,
}

/// A single variant in an enum type. Simple variants have an empty `fields` vec.
///
/// **Documentation:** `docs/pascal/05-types.md`
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariantTy {
    pub name: String,
    pub fields: Vec<(String, Ty)>,
}

impl EnumTy {
    /// True when at least one variant carries associated data.
    pub fn has_data(&self) -> bool {
        self.variants.iter().any(|v| !v.fields.is_empty())
    }

    /// Variant names as a plain list (for backwards-compatible helpers).
    pub fn member_names(&self) -> Vec<String> {
        self.variants.iter().map(|v| v.name.clone()).collect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionTy {
    pub params: Vec<ParamTy>,
    pub return_type: Box<Ty>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcedureTy {
    pub params: Vec<ParamTy>,
    /// Accept any number of arguments at the call site (e.g. `Std.Console.WriteLn`).
    pub variadic: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParamTy {
    pub mutable: bool,
    pub name: String,
    pub ty: Ty,
}

impl Ty {
    /// Returns true if this type is the error sentinel.
    pub fn is_error(&self) -> bool {
        matches!(self, Ty::Error)
    }

    /// Returns true if both types are compatible (same type or one is Error).
    pub fn compatible_with(&self, other: &Ty) -> bool {
        if self.is_error() || other.is_error() {
            return true;
        }
        match (self, other) {
            // GenericParam is compatible with anything (erased at runtime).
            (Ty::GenericParam(_), _) | (_, Ty::GenericParam(_)) => true,
            // Named type matches the concrete type with the same name (recursive enums).
            (Ty::Named(n), Ty::Enum(e)) | (Ty::Enum(e), Ty::Named(n)) => {
                n.eq_ignore_ascii_case(&e.name)
            }
            (Ty::Named(a), Ty::Named(b)) => a.eq_ignore_ascii_case(b),
            // Char widens to String and vice versa.
            (Ty::Char, Ty::String) | (Ty::String, Ty::Char) => true,
            // Array with Error element type is compatible with any array
            (Ty::Array(a), Ty::Array(b)) => a.compatible_with(b),
            // Records: structural compatibility (ignore name)
            (Ty::Record(a), Ty::Record(b)) => Self::record_fields_compatible(&a.fields, &b.fields),
            // Enums: same name is sufficient (type-erased generics).
            (Ty::Enum(a), Ty::Enum(b)) => a.name == b.name,
            (Ty::Unit, Ty::Unit) => true,
            // Result covariance
            (Ty::Result(ok1, err1), Ty::Result(ok2, err2)) => {
                ok1.compatible_with(ok2) && err1.compatible_with(err2)
            }
            // Option covariance
            (Ty::Option(a), Ty::Option(b)) => a.compatible_with(b),
            // Channel and Task covariance (inner type may be erased as Error)
            (Ty::Channel(a), Ty::Channel(b)) => a.compatible_with(b),
            (Ty::Task(a), Ty::Task(b)) => a.compatible_with(b),
            // Dict covariance
            (Ty::Dict(k1, v1), Ty::Dict(k2, v2)) => {
                k1.compatible_with(k2) && v1.compatible_with(v2)
            }
            _ => self == other,
        }
    }

    /// True for numeric types (integer, real).
    pub fn is_numeric(&self) -> bool {
        matches!(self, Ty::Integer | Ty::Real)
    }

    /// True for ordinal types (integer, boolean, char, simple enum without data).
    pub fn is_ordinal(&self) -> bool {
        match self {
            Ty::Integer | Ty::Boolean | Ty::Char => true,
            Ty::Enum(e) => !e.has_data(),
            _ => false,
        }
    }

    fn record_fields_compatible(fields: &[(String, Ty)], other_fields: &[(String, Ty)]) -> bool {
        if fields.len() != other_fields.len() {
            return false;
        }

        fields.iter().all(|(name, ty)| {
            other_fields
                .iter()
                .find(|(other_name, _)| other_name == name)
                .is_some_and(|(_, other_ty)| ty.compatible_with(other_ty))
        }) && other_fields.iter().all(|(name, ty)| {
            fields
                .iter()
                .find(|(other_name, _)| other_name == name)
                .is_some_and(|(_, other_ty)| ty.compatible_with(other_ty))
        })
    }
}
