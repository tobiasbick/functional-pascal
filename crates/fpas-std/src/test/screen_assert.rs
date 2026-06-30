//! Screen and view assertion helpers for `Std.Test` (VM-dispatched).
//!
//! **Documentation:** `docs/pascal/std/testing/test.md`

use super::assert::{assert_equals_integer, assert_equals_string, fail_with_message};
use crate::error::StdError;
use fpas_bytecode::SourceLocation;

/// Fail when `actual` row text differs from `expected` (trailing spaces ignored).
pub fn assert_screen_line(
    expected: String,
    actual: String,
    location: SourceLocation,
) -> Result<(), StdError> {
    assert_equals_string(
        expected.trim_end().to_string(),
        actual.trim_end().to_string(),
        location,
    )
}

/// Fail when a CRT cell differs from the expected character or packed colors.
pub fn assert_screen_cell(
    expected_ch: char,
    expected_fg: i64,
    expected_bg: i64,
    actual_ch: char,
    actual_fg: u8,
    actual_bg: u8,
    location: SourceLocation,
) -> Result<(), StdError> {
    if expected_ch != actual_ch {
        return fail_with_message(
            format!("test assertion failed: expected cell char '{expected_ch}', got '{actual_ch}'"),
            location,
        );
    }
    assert_equals_integer(expected_fg, i64::from(actual_fg), location)?;
    assert_equals_integer(expected_bg, i64::from(actual_bg), location)
}
