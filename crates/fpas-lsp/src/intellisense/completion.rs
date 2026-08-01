//! Rich completion items and lazy documentation resolution.

use fpas_language_service::{
    CompletionCandidate, CompletionDocumentation, CompletionKind, CompletionSource,
    DocumentSnapshot, SymbolKind,
};
use serde::{Deserialize, Serialize};
use tower_lsp_server::ls_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionTextEdit,
    Documentation, MarkupContent, MarkupKind, TextEdit, Uri,
};

use crate::convert::PositionConversionError;
use crate::navigation::span_range;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompletionResolveData {
    uri: String,
    declaration_offset: usize,
}

pub(crate) fn completion_item(
    snapshot: &DocumentSnapshot,
    value: CompletionCandidate,
) -> Result<CompletionItem, PositionConversionError> {
    let data = value
        .documentation
        .as_ref()
        .and_then(resolve_data)
        .and_then(|data| serde_json::to_value(data).ok());
    let additional_text_edits = value
        .additional_edit
        .map(|edit| {
            Ok(vec![TextEdit::new(
                span_range(snapshot, edit.span)?,
                edit.new_text,
            )])
        })
        .transpose()?;
    Ok(CompletionItem {
        label: value.label,
        label_details: Some(CompletionItemLabelDetails {
            detail: None,
            description: value.owner,
        }),
        kind: Some(completion_kind(value.kind)),
        detail: Some(match value.source {
            CompletionSource::AutoImport => format!("{} (auto import)", value.detail),
            CompletionSource::Declaration | CompletionSource::Keyword => value.detail,
        }),
        sort_text: Some(value.sort_text),
        filter_text: Some(value.filter_text),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit::new(
            span_range(snapshot, value.replacement_span)?,
            value.insert_text,
        ))),
        additional_text_edits,
        data,
        ..CompletionItem::default()
    })
}

pub(crate) fn resolve_completion_item(
    mut item: CompletionItem,
    documentation: Option<String>,
) -> CompletionItem {
    item.documentation = documentation.map(|value| {
        Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value,
        })
    });
    item
}

pub(crate) fn resolve_identity(item: &CompletionItem) -> Option<(std::path::PathBuf, usize)> {
    let data = serde_json::from_value::<CompletionResolveData>(item.data.clone()?).ok()?;
    let uri = data.uri.parse::<Uri>().ok()?;
    crate::convert::file_uri_to_path(&uri)
        .ok()
        .map(|path| (path, data.declaration_offset))
}

fn resolve_data(value: &CompletionDocumentation) -> Option<CompletionResolveData> {
    Some(CompletionResolveData {
        uri: Uri::from_file_path(&value.path)?.to_string(),
        declaration_offset: value.declaration_offset,
    })
}

fn completion_kind(kind: CompletionKind) -> CompletionItemKind {
    match kind {
        CompletionKind::Keyword => CompletionItemKind::KEYWORD,
        CompletionKind::Symbol(kind) => symbol_completion_kind(kind),
    }
}

fn symbol_completion_kind(kind: SymbolKind) -> CompletionItemKind {
    match kind {
        SymbolKind::Program => CompletionItemKind::FILE,
        SymbolKind::Unit => CompletionItemKind::MODULE,
        SymbolKind::Constant => CompletionItemKind::CONSTANT,
        SymbolKind::Variable
        | SymbolKind::MutableVariable
        | SymbolKind::Parameter
        | SymbolKind::LoopVariable => CompletionItemKind::VARIABLE,
        SymbolKind::Type => CompletionItemKind::CLASS,
        SymbolKind::Function | SymbolKind::Procedure => CompletionItemKind::FUNCTION,
        SymbolKind::Field => CompletionItemKind::FIELD,
        SymbolKind::Property => CompletionItemKind::PROPERTY,
        SymbolKind::Event => CompletionItemKind::EVENT,
        SymbolKind::EnumMember => CompletionItemKind::ENUM_MEMBER,
    }
}
