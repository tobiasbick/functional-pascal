//! Conversion from protocol-independent navigation results to LSP values.

mod references;
mod rename;
mod symbols;

use fpas_diagnostics::SourceSpan;
use fpas_language_service::{
    CompletionCandidate, DocumentSnapshot, HoverInfo, SymbolKind, SymbolLocation,
};
use tower_lsp_server::ls_types::{
    CompletionItem, CompletionItemKind, Hover, HoverContents, Location, MarkupContent, MarkupKind,
    Range, Uri,
};

use crate::convert::{PositionConversionError, byte_offset_to_position};

pub(crate) use references::reference_location;
pub(crate) use rename::{prepare_rename, rename_edit};
pub(crate) use symbols::document_symbols;

pub(crate) fn hover(
    snapshot: &DocumentSnapshot,
    value: HoverInfo,
) -> Result<Hover, PositionConversionError> {
    Ok(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("```pascal\n{}\n```", value.contents),
        }),
        range: Some(span_range(snapshot, value.range)?),
    })
}

pub(crate) fn location(
    snapshot: &DocumentSnapshot,
    value: &SymbolLocation,
) -> Result<Location, NavigationConversionError> {
    let uri = Uri::from_file_path(&value.path).ok_or(NavigationConversionError::InvalidPath)?;
    Ok(Location::new(
        uri,
        span_range(snapshot, value.symbol.selection_span)?,
    ))
}

pub(crate) fn completion(value: CompletionCandidate) -> CompletionItem {
    CompletionItem {
        label: value.label,
        kind: Some(completion_kind(value.kind)),
        detail: Some(value.detail),
        ..CompletionItem::default()
    }
}

pub(crate) fn span_range(
    snapshot: &DocumentSnapshot,
    span: SourceSpan,
) -> Result<Range, PositionConversionError> {
    let end = span.offset.saturating_add(span.length);
    Ok(Range::new(
        byte_offset_to_position(snapshot, span.offset)?,
        byte_offset_to_position(snapshot, end)?,
    ))
}

pub(crate) fn symbol_kind(kind: SymbolKind) -> tower_lsp_server::ls_types::SymbolKind {
    use tower_lsp_server::ls_types::SymbolKind as Lsp;
    match kind {
        SymbolKind::Program => Lsp::FILE,
        SymbolKind::Unit => Lsp::MODULE,
        SymbolKind::Constant => Lsp::CONSTANT,
        SymbolKind::Variable | SymbolKind::MutableVariable | SymbolKind::LoopVariable => {
            Lsp::VARIABLE
        }
        SymbolKind::Type => Lsp::CLASS,
        SymbolKind::Function => Lsp::FUNCTION,
        SymbolKind::Procedure => Lsp::FUNCTION,
        SymbolKind::Parameter => Lsp::VARIABLE,
        SymbolKind::Field => Lsp::FIELD,
        SymbolKind::Property => Lsp::PROPERTY,
        SymbolKind::Event => Lsp::EVENT,
        SymbolKind::EnumMember => Lsp::ENUM_MEMBER,
    }
}

fn completion_kind(kind: SymbolKind) -> CompletionItemKind {
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

#[derive(Debug)]
pub(crate) enum NavigationConversionError {
    InvalidPath,
    Position(PositionConversionError),
}

impl std::fmt::Display for NavigationConversionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath => formatter.write_str("source path cannot be represented as a URI"),
            Self::Position(error) => error.fmt(formatter),
        }
    }
}

impl From<PositionConversionError> for NavigationConversionError {
    fn from(error: PositionConversionError) -> Self {
        Self::Position(error)
    }
}
