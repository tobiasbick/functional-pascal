//! Conversion from protocol-independent navigation results to LSP values.

mod highlights;
mod references;
mod rename;
mod selection;
mod symbols;
mod workspace_symbols;

use fpas_diagnostics::SourceSpan;
use fpas_language_service::{DocumentSnapshot, HoverInfo, SymbolKind, SymbolLocation};
use tower_lsp_server::ls_types::{
    Hover, HoverContents, Location, LocationLink, MarkupContent, MarkupKind, Range, Uri,
};

use crate::convert::{PositionConversionError, byte_offset_to_position};

pub(crate) use references::reference_location;
pub(crate) use rename::{prepare_rename, rename_edit};
pub(crate) use selection::selection_range;
pub(crate) use symbols::document_symbols;
pub(crate) use workspace_symbols::workspace_symbol;

pub(crate) fn hover(
    snapshot: &DocumentSnapshot,
    value: HoverInfo,
) -> Result<Hover, PositionConversionError> {
    let mut markdown =
        crate::presentation::declaration_markdown(&value.contents, value.documentation.as_deref());
    if let Some(definition) = value.definition
        && let Some(uri) = Uri::from_file_path(&definition.path)
    {
        let position = byte_offset_to_position(&definition.snapshot, definition.span.offset())?;
        markdown.push_str(&format!(
            "\n\n[Go to definition](<{uri}#L{},{}>)",
            position.line + 1,
            position.character + 1,
            uri = uri.as_str(),
        ));
    }
    if let Some(handbook) = value.handbook {
        markdown.push_str(&format!(
            "\n\n[FPAS reference](https://github.com/tobiasbick/functional-pascal/blob/main/{handbook})"
        ));
    }
    Ok(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: markdown,
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

pub(crate) fn definition_link(
    snapshot: &DocumentSnapshot,
    value: &SymbolLocation,
) -> Result<LocationLink, NavigationConversionError> {
    Ok(LocationLink {
        origin_selection_range: None,
        target_uri: Uri::from_file_path(&value.path)
            .ok_or(NavigationConversionError::InvalidPath)?,
        target_range: span_range(snapshot, value.symbol.full_span)?,
        target_selection_range: span_range(snapshot, value.symbol.selection_span)?,
    })
}

pub(crate) fn span_range(
    snapshot: &DocumentSnapshot,
    span: SourceSpan,
) -> Result<Range, PositionConversionError> {
    let end = span.end();
    Ok(Range::new(
        byte_offset_to_position(snapshot, span.offset())?,
        byte_offset_to_position(snapshot, end)?,
    ))
}

/// Maps declarations and local import aliases to LSP symbol categories.
pub(crate) fn symbol_kind(kind: SymbolKind) -> tower_lsp_server::ls_types::SymbolKind {
    use tower_lsp_server::ls_types::SymbolKind as Lsp;
    match kind {
        SymbolKind::Program => Lsp::FILE,
        SymbolKind::Unit | SymbolKind::ImportAlias => Lsp::MODULE,
        SymbolKind::Constant => Lsp::CONSTANT,
        SymbolKind::Variable | SymbolKind::LoopVariable => Lsp::VARIABLE,
        SymbolKind::Type => Lsp::CLASS,
        SymbolKind::Enum => Lsp::ENUM,
        SymbolKind::Function => Lsp::FUNCTION,
        SymbolKind::Procedure => Lsp::FUNCTION,
        SymbolKind::Method => Lsp::METHOD,
        SymbolKind::TypeParameter => Lsp::TYPE_PARAMETER,
        SymbolKind::Parameter => Lsp::VARIABLE,
        SymbolKind::Field => Lsp::FIELD,
        SymbolKind::EnumMember => Lsp::ENUM_MEMBER,
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
pub(crate) use highlights::document_highlight;
