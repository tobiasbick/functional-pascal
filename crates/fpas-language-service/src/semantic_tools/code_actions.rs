//! Compiler-diagnostic validation and conservative source-edit gates.

use std::path::Path;

use fpas_diagnostics::codes::{SEMA_UNKNOWN_NAME, SEMA_UNKNOWN_TYPE};
use fpas_parser::parse_compilation_unit;

use super::{DiagnosticIdentity, SemanticCodeAction, SemanticEdit};
use crate::navigation::{NavigationResult, NavigationResult as ResultWithSnapshot};
use crate::{CompletionSource, LanguageService, LanguageServiceError};

impl LanguageService {
    /// Returns quick fixes authorized by one current compiler diagnostic.
    pub fn code_actions(
        &mut self,
        path: &Path,
        trigger: &DiagnosticIdentity,
    ) -> Result<NavigationResult<Vec<SemanticCodeAction>>, LanguageServiceError> {
        let analysis = self.analyze_document(path)?;
        let snapshot = analysis.snapshot().clone();
        let Some(current) = analysis.diagnostics().iter().find(|diagnostic| {
            diagnostic.code == trigger.code
                && diagnostic.message == trigger.message
                && diagnostic.span.offset == trigger.span.offset
                && diagnostic.span.length == trigger.span.length
                && diagnostic.span.source_id == 0
        }) else {
            return Ok(ResultWithSnapshot {
                snapshot,
                value: Vec::new(),
            });
        };
        let identity = DiagnosticIdentity {
            code: current.code,
            message: current.message.clone(),
            span: current.span,
        };
        let value = import_action(self, path, &snapshot, &identity)
            .into_iter()
            .collect();
        Ok(ResultWithSnapshot { snapshot, value })
    }
}

fn import_action(
    service: &mut LanguageService,
    path: &Path,
    snapshot: &crate::DocumentSnapshot,
    diagnostic: &DiagnosticIdentity,
) -> Option<SemanticCodeAction> {
    if !matches!(diagnostic.code, SEMA_UNKNOWN_NAME | SEMA_UNKNOWN_TYPE) {
        return None;
    }
    let name = diagnostic_name(&diagnostic.message)?;
    let identifier = identifier_in_span(snapshot.source(), diagnostic.span, name)?;
    let completions = service
        .completions(path, identifier.offset + identifier.length)
        .ok()?;
    let candidate = completions.value.into_iter().find(|candidate| {
        candidate.source == CompletionSource::AutoImport
            && candidate.label.eq_ignore_ascii_case(name)
    })?;
    let owner = candidate.owner?;
    let edit = candidate.additional_edit?;
    let semantic_edit = SemanticEdit {
        span: edit.span,
        new_text: edit.new_text,
    };
    canonical_after_edit(snapshot.source(), &semantic_edit).then(|| SemanticCodeAction {
        title: format!("Import {owner}"),
        diagnostic: diagnostic.clone(),
        edits: vec![semantic_edit],
    })
}

fn diagnostic_name(message: &str) -> Option<&str> {
    let start = message.find('`')? + 1;
    let end = message.get(start..)?.find('`')? + start;
    let name = message.get(start..end)?;
    let mut bytes = name.bytes();
    let first = bytes.next()?;
    ((first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|value| value.is_ascii_alphanumeric() || value == b'_'))
    .then_some(name)
}

fn identifier_in_span(
    source: &str,
    diagnostic: fpas_diagnostics::SourceSpan,
    name: &str,
) -> Option<fpas_diagnostics::SourceSpan> {
    let end = diagnostic
        .offset
        .saturating_add(diagnostic.length)
        .min(source.len());
    let fragment = source.get(diagnostic.offset..end)?;
    let relative = fragment.find(name)?;
    let offset = diagnostic.offset + relative;
    let before = source.as_bytes().get(offset.wrapping_sub(1)).copied();
    let after = source.as_bytes().get(offset + name.len()).copied();
    if before.is_some_and(identifier_byte) || after.is_some_and(identifier_byte) {
        return None;
    }
    Some(fpas_diagnostics::SourceSpan::new(
        offset,
        name.len(),
        diagnostic.line,
        diagnostic.column,
    ))
}

fn identifier_byte(value: u8) -> bool {
    value.is_ascii_alphanumeric() || value == b'_'
}

fn canonical_after_edit(source: &str, edit: &SemanticEdit) -> bool {
    let end = edit.span.offset.saturating_add(edit.span.length);
    if end > source.len()
        || !source.is_char_boundary(edit.span.offset)
        || !source.is_char_boundary(end)
    {
        return false;
    }
    let mut edited = source.to_owned();
    edited.replace_range(edit.span.offset..end, &edit.new_text);
    let (unit, diagnostics) = parse_compilation_unit(&edited);
    diagnostics.is_empty()
        && fpas_fmt::format_source(&edited, &unit).is_ok_and(|formatted| formatted == edited)
}
