//! Compiler-diagnostic validation and conservative source-edit gates.

use std::path::Path;

use fpas_diagnostics::codes::{SEMA_UNKNOWN_NAME, SEMA_UNKNOWN_TYPE, SEMA_UNQUALIFIED_NAME};
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
                && diagnostic.span.is_some_and(|span| {
                    span.offset() == trigger.span.offset()
                        && span.length() == trigger.span.length()
                        && span.source_id() == 0
                })
        }) else {
            return Ok(ResultWithSnapshot {
                snapshot,
                value: Vec::new(),
            });
        };
        let identity = DiagnosticIdentity {
            code: current.code,
            message: current.message.clone(),
            span: trigger.span,
        };
        let mut value = compiler_fix_actions(current, &identity);
        if value.is_empty() {
            value.extend(import_action(self, path, &snapshot, &identity));
        }
        Ok(ResultWithSnapshot { snapshot, value })
    }
}

fn import_action(
    service: &mut LanguageService,
    path: &Path,
    snapshot: &crate::DocumentSnapshot,
    diagnostic: &DiagnosticIdentity,
) -> Option<SemanticCodeAction> {
    if !matches!(
        diagnostic.code,
        SEMA_UNQUALIFIED_NAME | SEMA_UNKNOWN_NAME | SEMA_UNKNOWN_TYPE
    ) {
        return None;
    }
    let name = diagnostic_name(&diagnostic.message)?;
    let identifier = identifier_in_span(snapshot.source(), diagnostic.span, name)?;
    let completions = service.completions(path, identifier.end()).ok()?;
    let mut candidates = completions.value.into_iter().filter(|candidate| {
        (candidate.source == CompletionSource::AutoImport
            || (diagnostic.code == SEMA_UNQUALIFIED_NAME && candidate.insert_text.contains('.')))
            && candidate
                .insert_text
                .rsplit('.')
                .next()
                .is_some_and(|leaf| leaf.eq_ignore_ascii_case(name))
    });
    let candidate = candidates.next()?;
    if candidates.next().is_some() {
        return None;
    }
    let owner = candidate.owner?;
    let mut edits = vec![SemanticEdit {
        span: identifier,
        new_text: candidate.insert_text,
    }];
    if let Some(edit) = candidate.additional_edit {
        edits.push(SemanticEdit {
            span: edit.span,
            new_text: edit.new_text,
        });
    }
    canonical_after_edits(snapshot.source(), &edits).then(|| SemanticCodeAction {
        title: format!("Use qualified symbol from {owner}"),
        diagnostic: diagnostic.clone(),
        edits,
        preferred: true,
    })
}

/// Converts the compiler's own fixes of `current` into quick fixes; edits in another source
/// rule a fix out.
fn compiler_fix_actions(
    current: &fpas_diagnostics::Diagnostic,
    identity: &DiagnosticIdentity,
) -> Vec<SemanticCodeAction> {
    current
        .fixes
        .iter()
        .filter(|fix| fix.edits.iter().all(|edit| edit.span.source_id() == 0))
        .map(|fix| SemanticCodeAction {
            title: fix.title.clone(),
            diagnostic: identity.clone(),
            edits: fix
                .edits
                .iter()
                .map(|edit| SemanticEdit {
                    span: edit.span,
                    new_text: edit.new_text.clone(),
                })
                .collect(),
            preferred: fix.is_machine_applicable(),
        })
        .collect()
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
        .offset()
        .saturating_add(diagnostic.length())
        .min(source.len());
    let fragment = source.get(diagnostic.offset()..end)?;
    let relative = fragment.find(name)?;
    let offset = diagnostic.offset() + relative;
    let before = source.as_bytes().get(offset.wrapping_sub(1)).copied();
    let after = source.as_bytes().get(offset + name.len()).copied();
    if before.is_some_and(identifier_byte) || after.is_some_and(identifier_byte) {
        return None;
    }
    Some(fpas_diagnostics::SourceSpan::new(
        offset,
        name.len(),
        diagnostic.line(),
        diagnostic.column(),
    ))
}

fn identifier_byte(value: u8) -> bool {
    value.is_ascii_alphanumeric() || value == b'_'
}

fn canonical_after_edits(source: &str, edits: &[SemanticEdit]) -> bool {
    let mut edited = source.to_owned();
    let mut ordered = edits.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|edit| std::cmp::Reverse(edit.span.offset()));
    let mut previous = source.len();
    for edit in ordered {
        let end = edit.span.end();
        if end > previous
            || !source.is_char_boundary(edit.span.offset())
            || !source.is_char_boundary(end)
        {
            return false;
        }
        edited.replace_range(edit.span.offset()..end, &edit.new_text);
        previous = edit.span.offset();
    }
    let (unit, diagnostics) = parse_compilation_unit(&edited);
    diagnostics.is_empty()
        && fpas_fmt::format_source(&edited, &unit).is_ok_and(|formatted| formatted == edited)
}
