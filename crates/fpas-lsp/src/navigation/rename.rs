//! LSP conversion for prepare-rename selections and workspace text edits.

use fpas_language_service::{DocumentSnapshot, RenameEdit, RenameTarget, SourceVersion};
use tower_lsp_server::ls_types::{PrepareRenameResponse, TextEdit, Uri};

use super::{NavigationConversionError, span_range};

pub(crate) fn prepare_rename(
    snapshot: &DocumentSnapshot,
    target: RenameTarget,
) -> Result<PrepareRenameResponse, NavigationConversionError> {
    Ok(PrepareRenameResponse::RangeWithPlaceholder {
        range: span_range(snapshot, target.range)?,
        placeholder: target.placeholder,
    })
}

pub(crate) fn rename_edit(
    edit: RenameEdit,
) -> Result<(Uri, Option<i32>, TextEdit), NavigationConversionError> {
    let uri = Uri::from_file_path(&edit.path).ok_or(NavigationConversionError::InvalidPath)?;
    let version = match edit.snapshot.version() {
        SourceVersion::Editor(version) => i32::try_from(version).ok(),
        SourceVersion::Disk(_) => None,
    };
    Ok((
        uri,
        version,
        TextEdit::new(span_range(&edit.snapshot, edit.range)?, edit.new_text),
    ))
}
