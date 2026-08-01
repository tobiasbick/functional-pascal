//! Diagnostic identity parsing and single-document quick-fix conversion.

use std::collections::HashMap;

use fpas_diagnostics::{DiagnosticCode, SourceSpan};
use fpas_language_service::{DiagnosticIdentity, DocumentSnapshot, SemanticCodeAction};
use tower_lsp_server::ls_types::{
    CodeAction, CodeActionKind, Diagnostic, NumberOrString, TextEdit, Uri, WorkspaceEdit,
};

use crate::convert::{PositionConversionError, position_to_byte_offset};
use crate::navigation::span_range;

pub(crate) fn diagnostic_identity(
    snapshot: &DocumentSnapshot,
    diagnostic: &Diagnostic,
) -> Result<Option<DiagnosticIdentity>, PositionConversionError> {
    if diagnostic.source.as_deref() != Some("fpas") {
        return Ok(None);
    }
    let Some(NumberOrString::String(code)) = diagnostic.code.as_ref() else {
        return Ok(None);
    };
    let Some(value) = code
        .strip_prefix('F')
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|value| *value <= DiagnosticCode::MAX_VALUE)
    else {
        return Ok(None);
    };
    let start = position_to_byte_offset(snapshot, diagnostic.range.start)?;
    let end = position_to_byte_offset(snapshot, diagnostic.range.end)?;
    let Some(position) = snapshot.line_index().position(snapshot.source(), start) else {
        return Ok(None);
    };
    let message = diagnostic
        .message
        .split_once("\n\nHelp: ")
        .map_or(diagnostic.message.as_str(), |(message, _)| message);
    Ok(Some(DiagnosticIdentity {
        code: DiagnosticCode::new(value),
        message: message.to_owned(),
        span: SourceSpan::new(
            start,
            end.saturating_sub(start),
            u32::try_from(position.line + 1).unwrap_or(u32::MAX),
            u32::try_from(position.byte_column + 1).unwrap_or(u32::MAX),
        ),
    }))
}

pub(crate) fn code_action(
    snapshot: &DocumentSnapshot,
    uri: Uri,
    value: SemanticCodeAction,
    diagnostic: Diagnostic,
) -> Result<CodeAction, PositionConversionError> {
    let edits = value
        .edits
        .into_iter()
        .map(|edit| {
            Ok(TextEdit::new(
                span_range(snapshot, edit.span)?,
                edit.new_text,
            ))
        })
        .collect::<Result<Vec<_>, PositionConversionError>>()?;
    Ok(CodeAction {
        title: value.title,
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic]),
        edit: Some(WorkspaceEdit {
            changes: Some(HashMap::from([(uri, edits)])),
            ..WorkspaceEdit::default()
        }),
        is_preferred: Some(true),
        ..CodeAction::default()
    })
}
