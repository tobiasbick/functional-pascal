//! Exact inspection excerpts and explicit unavailable or deliberately omitted context.

use crate::{DeclarationOrigin, DocumentSnapshot};
use fpas_diagnostics::SourceSpan;
use std::sync::Arc;

/// Maximum aggregate bytes of source excerpts in one inspection.
pub const INSPECT_EXCERPT_BYTES: usize = 32 * 1024;
/// Maximum directly relevant supporting type declarations.
pub const INSPECT_SUPPORTING_TYPES: usize = 12;

/// One source-oriented inspection section.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionSectionKind {
    /// Declaration signature, contracts and attached documentation.
    Signature,
    /// Actual FPAS implementation of the selected routine.
    Implementation,
    /// A directly named supporting type or alias.
    SupportingType,
}

/// An exact continuous excerpt; separate excerpts must not be joined as continuous source.
pub struct SourceExcerpt {
    /// Exact range of the returned text.
    pub span: SourceSpan,
    /// Original source bytes, including line endings.
    pub text: String,
}

/// A bounded view of a declaration or body in its original source.
pub struct InspectionSection {
    /// Presentation role and stable ordering category.
    pub kind: InspectionSectionKind,
    /// Why this section is relevant to the selected declaration.
    pub reason: &'static str,
    /// Exact semantic owner when physically present.
    pub owner: Option<String>,
    /// Declaration's canonical name.
    pub qualified_name: String,
    /// Source category; generated declarations are never implementation bodies.
    pub origin: DeclarationOrigin,
    /// Whether the facts were checked by the compiler.
    pub checked: bool,
    /// Immutable defining source, absent for source-less builtins.
    pub snapshot: Option<Arc<DocumentSnapshot>>,
    /// Full source range, independently of excerpt clipping.
    pub span: Option<SourceSpan>,
    /// SHA-256 of the complete defining file, not the excerpt.
    pub source_sha256: Option<String>,
    /// Original declaration signature or compiler-owned callable detail.
    pub detail: Option<String>,
    /// Attached documentation, absent when none exists.
    pub documentation: Option<String>,
    /// Bounded exact source pieces in source order.
    pub excerpts: Vec<SourceExcerpt>,
    /// Full byte size of this section before omission.
    pub full_bytes: usize,
    /// Bytes omitted for budgets or deliberate context exclusions.
    pub omitted_bytes: usize,
}

/// An unavailable or deliberately excluded part of inspection context.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct InspectionOmission {
    /// Requested section category.
    pub section: String,
    /// Stable explanation, separate from global analysis coverage.
    pub reason: String,
    /// Exact omitted bytes when known, otherwise absent.
    pub bytes: Option<usize>,
}

/// Declaration, implementation and one-hop supporting context in fixed priority order.
pub struct Inspection {
    /// Signature first, then implementation, then supporting types.
    pub sections: Vec<InspectionSection>,
    /// Missing bodies, bounded text and deliberately excluded deeper context.
    pub omissions: Vec<InspectionOmission>,
    /// False when required compiler bindings could not be recovered.
    pub analysis_complete: bool,
    /// Aggregate returned source bytes.
    pub excerpt_bytes: usize,
    /// Number of retained directly named type declarations.
    pub supporting_types: usize,
}
