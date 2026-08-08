//! Deterministic bounded register-object payload encoding.

use crate::object::{ObjectError, RelocatableObject};

/// Encode and validate a register object deterministically.
///
/// # Errors
///
/// Returns [`ObjectError`] for invalid or oversized object data.
pub fn encode_object(object: &RelocatableObject) -> Result<Vec<u8>, ObjectError> {
    object.validate()?;
    let bytes =
        serde_json::to_vec(object).map_err(|error| ObjectError::Encode(error.to_string()))?;
    check_payload_size(bytes.len())?;
    Ok(bytes)
}

/// Decode and validate a register object.
///
/// # Errors
///
/// Returns [`ObjectError`] for malformed, incompatible, invalid, or oversized object data.
pub fn decode_object(bytes: &[u8]) -> Result<RelocatableObject, ObjectError> {
    check_payload_size(bytes.len())?;
    let object =
        serde_json::from_slice(bytes).map_err(|error| ObjectError::Decode(error.to_string()))?;
    RelocatableObject::validate(&object)?;
    Ok(object)
}

fn check_payload_size(size: usize) -> Result<(), ObjectError> {
    crate::format::check_payload_size("register object", size).map_err(|_| {
        ObjectError::PayloadSize {
            size,
            maximum: crate::format::MAX_PAYLOAD_BYTES,
        }
    })
}
