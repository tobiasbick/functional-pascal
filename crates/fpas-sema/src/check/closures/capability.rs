//! Capture transfer capabilities. See `docs/pascal/language/functions/closures.md`.

use super::CaptureBinding;
use crate::TransferCapability;

/// Mutable captures retain local cells; immutable captures preserve their value capability.
#[must_use]
pub fn capability_from_captures(captures: &[CaptureBinding]) -> TransferCapability {
    captures
        .iter()
        .fold(TransferCapability::Transferable, |state, capture| {
            state.join(if capture.mutable {
                TransferCapability::TaskBound
            } else {
                capture.capability
            })
        })
}
