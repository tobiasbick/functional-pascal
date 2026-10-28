//! Source-visible distinct type names retained for debugger conversions.
//! See `docs/pascal/language/types/distinct-types.md` and `docs/pascal/tools/debugger.md`.

use crate::{DebugTypeId, SourceId, StringId};

/// One distinct type name visible in a source scope and its scalar underlying type.
///
/// Distinct values use their underlying runtime representation; this table only lets
/// debugger evaluation resolve `UserId(Value)` conversions by source-visible name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DistinctTypeName {
    /// Source containing the declaration, type alias, or import.
    pub source: SourceId,
    /// Source-visible type name, including a uses alias when applicable.
    pub name: StringId,
    /// Scalar `integer`, `real`, `string`, or `boolean` debug type.
    pub underlying: DebugTypeId,
}
