//! Recursive selection-range conversion.

use fpas_language_service::{DocumentSnapshot, SelectionRange as ServiceSelectionRange};
use tower_lsp_server::ls_types::SelectionRange;

use crate::convert::PositionConversionError;

use super::span_range;

pub(crate) fn selection_range(
    snapshot: &DocumentSnapshot,
    range: ServiceSelectionRange,
) -> Result<SelectionRange, PositionConversionError> {
    Ok(SelectionRange {
        range: span_range(snapshot, range.span)?,
        parent: range
            .parent
            .map(|parent| selection_range(snapshot, *parent).map(Box::new))
            .transpose()?,
    })
}
