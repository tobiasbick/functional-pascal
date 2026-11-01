//! Inspection construction bounds retained text before allocating excerpt strings.

use super::model::*;
use crate::{DeclarationOrigin, DocumentSnapshot, DocumentSymbol, identity_path, source_digest};
use fpas_diagnostics::SourceSpan;
use std::{ops::Range, sync::Arc};

pub(super) struct SourceSection {
    pub kind: InspectionSectionKind,
    pub reason: &'static str,
    pub origin: DeclarationOrigin,
    pub owner: Option<String>,
}

impl Inspection {
    pub(super) fn new() -> Self {
        Self {
            sections: Vec::new(),
            omissions: Vec::new(),
            analysis_complete: true,
            excerpt_bytes: 0,
            supporting_types: 0,
        }
    }
    pub(super) fn omit(
        &mut self,
        section: &'static str,
        reason: &'static str,
        bytes: Option<usize>,
    ) {
        if !self
            .omissions
            .iter()
            .any(|o| o.section == section && o.reason == reason && o.bytes == bytes)
        {
            self.omissions.push(InspectionOmission {
                section: section.into(),
                reason: reason.into(),
                bytes,
            });
        }
    }
    pub(super) fn source(
        &mut self,
        snapshot: &Arc<DocumentSnapshot>,
        symbol: &DocumentSymbol,
        span: SourceSpan,
        section: SourceSection,
        mut excluded: Vec<Range<usize>>,
    ) {
        let SourceSection {
            kind,
            reason,
            origin,
            owner,
        } = section;
        let source = snapshot.source();
        if source.get(span.offset()..span.end()).is_none() {
            self.analysis_complete = false;
            self.omit("source", "invalid_source_range", None);
            return;
        }
        excluded.sort_by_key(|r| r.start);
        let mut next = span.offset();
        let mut excerpts = Vec::new();
        let mut excluded_bytes = 0;
        let mut retained = 0;
        let mut available = INSPECT_EXCERPT_BYTES.saturating_sub(self.excerpt_bytes);
        let mut append = |range: Range<usize>| {
            let mut end = range.end.min(range.start.saturating_add(available));
            while end > range.start && !source.is_char_boundary(end) {
                end -= 1;
            }
            if end == range.start {
                return;
            }
            if let Some(span) = exact_span(snapshot, range.start..end, span.source_id()) {
                let bytes = end - range.start;
                available -= bytes;
                retained += bytes;
                excerpts.push(SourceExcerpt {
                    span,
                    text: source[range.start..end].into(),
                });
            }
        };
        for excluded in excluded {
            let start = excluded.start.max(next).min(span.end());
            let end = excluded.end.max(start).min(span.end());
            append(next..start);
            excluded_bytes += end - start;
            next = end;
        }
        append(next..span.end());
        self.excerpt_bytes += retained;
        let omitted = span.length() - retained;
        if excluded_bytes > 0 {
            self.omit(
                "supporting_types",
                "method_bodies_excluded",
                Some(excluded_bytes),
            );
        }
        if omitted > excluded_bytes {
            self.omit("source", "excerpt_limit", Some(omitted - excluded_bytes));
        }
        self.sections.push(InspectionSection {
            kind,
            reason,
            owner,
            qualified_name: symbol.qualified_name.clone(),
            origin,
            checked: true,
            snapshot: Some(Arc::clone(snapshot)),
            span: Some(span),
            source_sha256: Some(source_digest(source.as_bytes())),
            detail: (kind == InspectionSectionKind::Signature).then(|| {
                (origin == DeclarationOrigin::IntrinsicApi)
                    .then(|| super::super::intrinsic::detail(&symbol.qualified_name))
                    .flatten()
                    .unwrap_or_else(|| symbol.detail.clone())
            }),
            documentation: (kind != InspectionSectionKind::Implementation)
                .then(|| {
                    crate::documentation::preceding_documentation(source, symbol.full_span.offset())
                })
                .flatten(),
            excerpts,
            full_bytes: span.length(),
            omitted_bytes: omitted,
        });
    }
}

pub(super) fn exact_span(
    snapshot: &DocumentSnapshot,
    range: Range<usize>,
    source_id: u32,
) -> Option<SourceSpan> {
    let position = snapshot.line_index().position(range.start)?;
    Some(SourceSpan::new_with_source(
        range.start,
        range.end.checked_sub(range.start)?,
        u32::try_from(position.line + 1).ok()?,
        u32::try_from(position.byte_column + 1).ok()?,
        source_id,
    ))
}

pub(super) fn owner(service: &crate::LanguageService, path: &std::path::Path) -> Option<String> {
    if service.is_editor_api_source(path) {
        return None;
    }
    if service
        .standard_library_source_paths()
        .contains(&path.to_path_buf())
    {
        return service
            .standard_library_root()
            .and_then(|root| identity_path(&root.join("stdlib.fpasprj")).ok());
    }
    let owners = service
        .workspace()
        .projects()
        .iter()
        .filter(|p| p.contains_source(path))
        .filter_map(|p| match p.source_origin(path) {
            fpas_project::SourceOrigin::Own => identity_path(p.manifest_path()).ok(),
            fpas_project::SourceOrigin::Library(path) => identity_path(&path).ok(),
        })
        .collect::<std::collections::BTreeSet<_>>();
    (owners.len() == 1)
        .then(|| owners.into_iter().next())
        .flatten()
}

pub(super) fn origin(
    service: &crate::LanguageService,
    path: &std::path::Path,
) -> DeclarationOrigin {
    if service.is_editor_api_source(path) {
        DeclarationOrigin::IntrinsicApi
    } else if service
        .standard_library_source_paths()
        .contains(&path.to_path_buf())
    {
        DeclarationOrigin::StandardLibrary
    } else if service
        .workspace()
        .projects()
        .iter()
        .any(|p| p.owns_source(path))
    {
        DeclarationOrigin::Owned
    } else {
        DeclarationOrigin::Dependency
    }
}
