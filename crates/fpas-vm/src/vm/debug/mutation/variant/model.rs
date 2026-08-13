//! Public and crate-local models for variant discovery and construction.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

use std::sync::Arc;

use fpas_bytecode::{DebugTypeId, EnumVariantId, RuntimeEnumLayout};

use super::super::super::evaluation::DebugEvaluateResult;

/// Read-only description of every constructible variant on one wrapper type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugVariantDescription {
    /// Canonical wrapper type name, such as `Choice`, `Result`, or `Option`.
    pub type_name: String,
    /// Variants in executable metadata order.
    pub variants: Vec<DebugVariantInfo>,
}

/// One constructible enum, `Result`, or `Option` variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugVariantInfo {
    /// Canonical request name, such as `Choice.Pair` or `Ok`.
    pub name: String,
    /// Declared payload fields in declaration order.
    pub fields: Vec<DebugVariantField>,
}

/// One declared payload field required by complete construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugVariantField {
    /// Canonical field name from executable metadata.
    pub name: String,
    /// Portable display name of the declared field type.
    pub type_name: String,
}

/// Rendered result of one atomic complete-variant construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugVariantConstructionResult {
    /// Fresh retained summary of the committed wrapper value.
    pub value: DebugEvaluateResult,
    /// Canonical selected variant name.
    pub variant: String,
}

/// Normalized wrapper metadata used by discovery, construction, and transition.
#[derive(Clone)]
pub(in crate::vm::debug) struct WrapperMetadata {
    pub type_name: String,
    pub variants: Vec<VariantMetadata>,
}

/// One metadata-backed variant, including layouts needed to build a complete value.
#[derive(Clone)]
pub(in crate::vm::debug) struct VariantMetadata {
    /// Canonical protocol name (`Choice.Empty`, `Ok`, `None`).
    pub canonical_name: String,
    /// Unqualified metadata name used by qualified transition suffixes.
    pub name: String,
    pub fields: Vec<VariantFieldMetadata>,
    pub kind: VariantKind,
    pub variant_id: Option<EnumVariantId>,
}

/// Declared payload field with its portable debug type.
#[derive(Clone)]
pub(in crate::vm::debug) struct VariantFieldMetadata {
    pub name: String,
    pub ty: DebugTypeId,
    pub type_name: String,
}

/// Complete runtime construction strategy for one variant.
#[derive(Clone)]
pub(in crate::vm::debug) enum VariantKind {
    /// Data-enum variant, including fieldless and multi-field shapes.
    Enum {
        /// Runtime layout used to build the detached enum value.
        layout: Arc<RuntimeEnumLayout>,
    },
    /// `Result.Ok` wrapper.
    ResultOk,
    /// `Result.Error` wrapper.
    ResultError,
    /// `Option.Some` wrapper.
    OptionSome,
    /// `Option.None` wrapper.
    OptionNone,
}

impl WrapperMetadata {
    pub(in crate::vm::debug) fn description(&self) -> DebugVariantDescription {
        DebugVariantDescription {
            type_name: self.type_name.clone(),
            variants: self
                .variants
                .iter()
                .map(|variant| DebugVariantInfo {
                    name: variant.canonical_name.clone(),
                    fields: variant
                        .fields
                        .iter()
                        .map(|field| DebugVariantField {
                            name: field.name.clone(),
                            type_name: field.type_name.clone(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    pub(in crate::vm::debug) fn find_canonical<'a>(
        &'a self,
        name: &str,
    ) -> Result<&'a VariantMetadata, Vec<&'a VariantMetadata>> {
        let matches = self
            .variants
            .iter()
            .filter(|variant| variant.canonical_name.eq_ignore_ascii_case(name))
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [variant] => Ok(*variant),
            _ => Err(matches),
        }
    }

    pub(in crate::vm::debug) fn find_short<'a>(
        &'a self,
        name: &str,
    ) -> Result<&'a VariantMetadata, Vec<&'a VariantMetadata>> {
        let matches = self
            .variants
            .iter()
            .filter(|variant| variant.name.eq_ignore_ascii_case(name))
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [variant] => Ok(*variant),
            _ => Err(matches),
        }
    }
}

impl VariantMetadata {
    pub(in crate::vm::debug) fn field_names(&self) -> Vec<&str> {
        self.fields
            .iter()
            .map(|field| field.name.as_str())
            .collect()
    }

    pub(in crate::vm::debug) fn payload_type(&self) -> Option<DebugTypeId> {
        match self.fields.as_slice() {
            [field] => Some(field.ty),
            _ => None,
        }
    }
}
