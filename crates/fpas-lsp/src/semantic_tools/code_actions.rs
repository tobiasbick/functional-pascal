//! Diagnostic identity parsing and single-document quick-fix conversion.

use fpas_diagnostics::{DiagnosticCode, SourceSpan};
use fpas_language_service::{
    DiagnosticIdentity, DocumentSnapshot, SemanticCodeAction, SourceVersion,
};
use tower_lsp_server::ls_types::{
    CodeAction, CodeActionKind, Diagnostic, DocumentChanges, NumberOrString, OneOf,
    OptionalVersionedTextDocumentIdentifier, TextDocumentEdit, TextEdit, Uri, WorkspaceEdit,
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
    let Some(code) = code
        .strip_prefix('F')
        .and_then(|value| value.parse::<u16>().ok())
        .and_then(|value| DiagnosticCode::try_new(value).ok())
    else {
        return Ok(None);
    };
    let start = position_to_byte_offset(snapshot, diagnostic.range.start)?;
    let end = position_to_byte_offset(snapshot, diagnostic.range.end)?;
    let Some(position) = snapshot.line_index().position(start) else {
        return Ok(None);
    };
    let message = diagnostic
        .message
        .split_once("\n\nHelp: ")
        .map_or(diagnostic.message.as_str(), |(message, _)| message);
    Ok(Some(DiagnosticIdentity {
        code,
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
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri,
                    version: match snapshot.version() {
                        SourceVersion::Editor(version) => i32::try_from(version).ok(),
                        SourceVersion::Disk(_) => None,
                    },
                },
                edits: edits.into_iter().map(OneOf::Left).collect(),
            }])),
            ..WorkspaceEdit::default()
        }),
        is_preferred: Some(true),
        ..CodeAction::default()
    })
}
