//! Source-local unit imports and optional aliases.
//!
//! **Documentation:** `docs/pascal/program-structure/units.md`

use fpas_lexer::Span;

use super::QualifiedId;

/// One direct unit import in a `uses` clause.
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    /// Unit path identifying the imported unit, independent of its source-local alias.
    pub unit: QualifiedId,
    /// Optional source-local namespace for the unit's public symbols.
    pub alias: Option<ImportAlias>,
    /// Span covering the unit name and optional alias.
    pub span: Span,
}

/// An import alias and its exact identifier span.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportAlias {
    /// Alias spelling from source.
    pub name: String,
    /// Span of the alias identifier.
    pub span: Span,
}

impl From<QualifiedId> for Import {
    fn from(unit: QualifiedId) -> Self {
        Self {
            span: unit.span,
            unit,
            alias: None,
        }
    }
}
