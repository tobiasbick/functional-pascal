mod calls;
pub(crate) mod closures;
mod context;
mod decl;
mod entry;
mod expr;
mod name_resolution;
mod patterns;
mod purity;
mod record_visibility;
mod stmt;

/// Metadata for checked callable-value invocations.
pub use calls::{ValueCallMap, ValueCallTarget};
pub use closures::CaptureBinding;
pub use closures::ClosureInfo;
pub use closures::ClosureInfoMap;
pub use closures::NestedRoutineCaptureInfo;
pub use closures::NestedRoutineCaptureMap;
pub use context::AnalysisMetadata;
pub use context::BindingTypeMap;
pub(crate) use context::Checker;
pub use context::ExprTypeMap;
pub use context::IntrinsicCallMap;
pub use context::NamedTypeMap;
pub use context::RecordDefault;
pub use context::RecordDefaultsMap;
/// Concrete metadata for recursive case patterns.
pub use patterns::{PatternInfo, PatternInfoMap, PatternVariant};
