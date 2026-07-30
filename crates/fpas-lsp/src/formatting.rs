//! Conversion of canonical Functional Pascal formatting into LSP text edits.

use tower_lsp_server::ls_types::{Range, TextEdit};

use crate::convert::{PositionConversionError, byte_offset_to_position};
use crate::documents::FormattedDocument;

pub(crate) fn whole_document_edit(
    formatted: FormattedDocument,
) -> Result<Vec<TextEdit>, PositionConversionError> {
    if formatted.snapshot.source() == formatted.text {
        return Ok(Vec::new());
    }
    let end = byte_offset_to_position(&formatted.snapshot, formatted.snapshot.source().len())?;
    Ok(vec![TextEdit {
        range: Range::new(tower_lsp_server::ls_types::Position::new(0, 0), end),
        new_text: formatted.text,
    }])
}
