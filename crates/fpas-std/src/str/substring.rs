//! Scalar-indexed substring extraction without copying the complete input.

use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::{SharedStr, SourceLocation};
use fpas_diagnostics::codes::RUNTIME_STRING_INDEX_OUT_OF_BOUNDS;

/// Extracts a checked Unicode scalar range while retaining immutable value semantics.
///
/// Documentation: `docs/pascal/std/text/str/search.md`.
pub(super) fn extract(
    source: &SharedStr,
    start: i64,
    len: i64,
    location: SourceLocation,
) -> Result<SharedStr, StdError> {
    let count = source.char_len() as i64;
    if start < 0 || len < 0 || start > count || len > count - start {
        return Err(std_runtime_error(
            RUNTIME_STRING_INDEX_OUT_OF_BOUNDS,
            format!("Substring out of range (len={count}, start={start}, len_param={len})"),
            "Ensure `start` and `len` select a valid substring range.",
            location,
        ));
    }
    if start == 0 && len == count {
        return Ok(source.clone());
    }
    if len == 0 {
        return Ok("".into());
    }
    let start = start as usize;
    let len = len as usize;
    // Equal byte and scalar counts prove ASCII without another input scan.
    let (first, end) = if source.len() == source.char_len() {
        (start, start + len)
    } else {
        let mut boundaries = source.char_indices().map(|(offset, _)| offset);
        let first = boundaries.nth(start).unwrap_or(source.len());
        let end = boundaries.nth(len - 1).unwrap_or(source.len());
        (first, end)
    };
    Ok(source[first..end].into())
}
