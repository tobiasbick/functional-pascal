//! LSP conversion for Functional Pascal reference locations.

use fpas_language_service::ReferenceLocation;
use tower_lsp_server::ls_types::{Location, Uri};

use super::{NavigationConversionError, span_range};

pub(crate) fn reference_location(
    location: &ReferenceLocation,
) -> Result<Location, NavigationConversionError> {
    let uri = Uri::from_file_path(&location.path).ok_or(NavigationConversionError::InvalidPath)?;
    Ok(Location::new(
        uri,
        span_range(&location.snapshot, location.span)?,
    ))
}
