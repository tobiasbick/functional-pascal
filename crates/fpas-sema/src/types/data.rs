//! Nominal record and enum descriptors.

use super::{GenericParamDef, Ty};

/// Resolved record shape and field ownership.
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
