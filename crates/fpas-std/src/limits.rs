//! Runtime collection and parsing limits for `Std.*` intrinsics.
//!
//! **Documentation:** `docs/pascal/std/README.md`

use crate::error::{StdError, std_runtime_error};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS;

/// Maximum number of elements allowed in one `Std.Array.Fill` or `Std.Str.RepeatStr` result.
pub(crate) const MAX_COLLECTION_LEN: i64 = 1_000_000;

/// Maximum nesting depth accepted by `Std.Json.Parse` and `Std.Json.Stringify`.
pub(crate) const MAX_JSON_DEPTH: usize = 256;

/// Maximum nesting depth accepted by `Std.Toml.Parse` and `Std.Toml.Stringify`.
pub(crate) const MAX_TOML_DEPTH: usize = 256;

/// Validates a non-negative collection length against [`MAX_COLLECTION_LEN`].
pub(crate) fn checked_collection_len(
    count: i64,
    location: SourceLocation,
    api: &str,
) -> Result<usize, StdError> {
    if count < 0 {
        return Err(std_runtime_error(
            RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
            format!("{api} count must be >= 0, got {count}"),
            format!("Pass a non-negative integer to {api}."),
            location,
        ));
    }
    if count > MAX_COLLECTION_LEN {
        return Err(std_runtime_error(
            RUNTIME_ARRAY_INDEX_OUT_OF_BOUNDS,
            format!("{api} count {count} exceeds maximum allowed {MAX_COLLECTION_LEN}"),
            format!("Request at most {MAX_COLLECTION_LEN} elements from {api}."),
            location,
        ));
    }
    Ok(count as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loc() -> SourceLocation {
        SourceLocation::new(1, 1)
    }

    #[test]
    fn checked_collection_len_accepts_zero_and_max() {
        assert_eq!(checked_collection_len(0, loc(), "Test").unwrap(), 0);
        assert_eq!(
            checked_collection_len(MAX_COLLECTION_LEN, loc(), "Test").unwrap(),
            MAX_COLLECTION_LEN as usize
        );
    }

    #[test]
    fn checked_collection_len_rejects_negative_and_overflow() {
        assert!(checked_collection_len(-1, loc(), "Test").is_err());
        assert!(checked_collection_len(MAX_COLLECTION_LEN + 1, loc(), "Test").is_err());
    }
}
