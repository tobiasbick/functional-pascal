//! LSP conversion for Functional Pascal reference locations.

use fpas_language_service::{DocumentSnapshot, ReferenceLocation};
use tower_lsp_server::ls_types::{Location, Uri};

use super::{NavigationConversionError, span_range};

pub(crate) fn reference_location(
    snapshot: &DocumentSnapshot,
    location: &ReferenceLocation,
) -> Result<Location, NavigationConversionError> {
    let uri = Uri::from_file_path(&location.path).ok_or(NavigationConversionError::InvalidPath)?;
    Ok(Location::new(uri, span_range(snapshot, location.span)?))
}
