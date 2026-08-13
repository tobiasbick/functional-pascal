//! Metadata-driven discovery and complete construction of enum, Result, and Option variants.
//!
//! **Documentation:** `docs/pascal/tools/debugger.md`

mod construct;
mod diagnostics;
mod metadata;
mod model;

pub use model::{
    DebugVariantConstructionResult, DebugVariantDescription, DebugVariantField, DebugVariantInfo,
};

pub(in crate::vm::debug) use construct::{
    complete_value, constructible_description, ordered_field_expressions,
    require_constructible_fields,
};
pub(in crate::vm::debug) use diagnostics::{
    constructor_example, qualified_example, unknown_variant, unsupported_metadata,
};
pub(in crate::vm::debug) use metadata::{require_wrapper, try_wrapper};
pub(in crate::vm::debug) use model::{VariantKind, VariantMetadata, WrapperMetadata};
