//! One typed inspection wire shape, parameterized only by adapter source locations.

use super::{Inspection, InspectionOmission, InspectionSectionKind};
use crate::{DeclarationOrigin, DocumentSnapshot};
use fpas_diagnostics::SourceSpan;
use serde::{Deserialize, Serialize};

/// Inspection facts with CLI scalar or LSP UTF-16 locations supplied by the adapter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InspectionProjection<L> {
    /// Ordered declaration, implementation and supporting type sections.
    pub sections: Vec<InspectionSectionProjection<L>>,
    /// Unavailable, deliberately excluded or budget-clipped context.
    pub omissions: Vec<InspectionOmission>,
    /// Aggregate bytes of exact retained source excerpts.
    pub excerpt_bytes: usize,
    /// Number of directly named supporting declarations.
    pub supporting_types: usize,
}

impl<L> Default for InspectionProjection<L> {
    fn default() -> Self {
        Self {
            sections: Vec::new(),
            omissions: Vec::new(),
            excerpt_bytes: 0,
            supporting_types: 0,
        }
    }
}

/// A section with exact identity fields and bounded presentation text.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InspectionSectionProjection<L> {
    /// Section purpose.
    pub kind: InspectionSectionKind,
    /// Reason the section belongs to this inspection.
    pub reason: String,
    /// Owning project when present.
    pub owner: Option<String>,
    /// Canonical declaration name.
    pub qualified_name: String,
    /// Defining source provenance.
    pub origin: DeclarationOrigin,
    /// Compiler checking state.
    pub checked: bool,
    /// Full source range, separate from the retained excerpts.
    pub location: Option<L>,
    /// Complete defining file digest.
    pub source_sha256: Option<String>,
    /// Bounded signature text.
    pub detail: Option<String>,
    /// Bounded attached documentation.
    pub documentation: Option<String>,
    /// Continuous exact source pieces; gaps remain explicit.
    pub excerpts: Vec<InspectionExcerptProjection<L>>,
    /// Full source bytes in the section.
    pub full_bytes: usize,
    /// Source bytes excluded by budget or scope.
    pub omitted_bytes: usize,
}

/// One continuous excerpt and the exact source range of its text.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InspectionExcerptProjection<L> {
    /// Exact location of the returned source bytes.
    pub location: L,
    /// Unmodified source text.
    pub text: String,
}

impl Inspection {
    /// Converts only locations and bounded text; identities and excerpt bytes remain exact.
    pub fn project<L, E>(
        &self,
        mut location: impl FnMut(&DocumentSnapshot, SourceSpan) -> Result<L, E>,
    ) -> Result<InspectionProjection<L>, E> {
        let mut omissions = self.omissions.clone();
        let mut text_omitted = 0;
        let mut clip = |text: &Option<String>| {
            text.as_ref().map(|text| {
                let clipped = text.chars().take(4096).collect::<String>();
                text_omitted += text.len() - clipped.len();
                clipped
            })
        };
        let mut sections = Vec::with_capacity(self.sections.len());
        for section in &self.sections {
            let mut excerpts = Vec::with_capacity(section.excerpts.len());
            if let Some(snapshot) = &section.snapshot {
                for excerpt in &section.excerpts {
                    excerpts.push(InspectionExcerptProjection {
                        location: location(snapshot, excerpt.span)?,
                        text: excerpt.text.clone(),
                    });
                }
            }
            sections.push(InspectionSectionProjection {
                kind: section.kind,
                reason: section.reason.into(),
                owner: section.owner.clone(),
                qualified_name: section.qualified_name.clone(),
                origin: section.origin,
                checked: section.checked,
                location: section
                    .snapshot
                    .as_ref()
                    .zip(section.span)
                    .map(|(s, span)| location(s, span))
                    .transpose()?,
                source_sha256: section.source_sha256.clone(),
                detail: clip(&section.detail),
                documentation: clip(&section.documentation),
                excerpts,
                full_bytes: section.full_bytes,
                omitted_bytes: section.omitted_bytes,
            });
        }
        if text_omitted > 0 {
            omissions.push(InspectionOmission {
                section: "documentation_signature".into(),
                reason: "text_limit".into(),
                bytes: Some(text_omitted),
            });
        }
        Ok(InspectionProjection {
            sections,
            omissions,
            excerpt_bytes: self.excerpt_bytes,
            supporting_types: self.supporting_types,
        })
    }
}
