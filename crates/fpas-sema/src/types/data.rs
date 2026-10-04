//! Nominal record and enum descriptors.

use super::{FunctionTy, GenericParamDef, ProcedureTy, Ty};

/// Resolved record shape, ownership, members, and callable metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordTy {
    /// Case-preserving qualified record name.
    pub name: String,
    /// Generic parameters declared by this nominal record.
    pub type_params: Vec<GenericParamDef>,
    /// Concrete or enclosing-generic arguments of an instantiated record.
    pub type_args: Vec<Ty>,
    /// Whether this nominal type designates an opaque host resource rather than value data.
    pub is_resource: bool,
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
    /// Generic parameters declared by this nominal enum.
    pub type_params: Vec<GenericParamDef>,
    /// Concrete or enclosing-generic arguments of an instantiated enum.
    pub type_args: Vec<Ty>,
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
