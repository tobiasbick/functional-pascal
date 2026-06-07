//! Assertion failure helpers for `Std.Test`.
//!
//! **Documentation:** `docs/pascal/std/test.md` (from the repository root).

use crate::error::{StdError, std_runtime_error_opt};
use fpas_bytecode::SourceLocation;
use fpas_diagnostics::codes::RUNTIME_TEST_ASSERTION_FAILED;

/// Fail the current test with a formatted message at `location`.
pub(crate) fn fail_with_message(
    message: impl Into<String>,
    location: SourceLocation,
) -> Result<(), StdError> {
    Err(std_runtime_error_opt(
        RUNTIME_TEST_ASSERTION_FAILED,
        message.into(),
        Some("Fix the failing condition or update the expected value in the test.".into()),
        location,
    ))
}

/// `AssertTrue`: fail when `cond` is false.
pub(crate) fn assert_true(cond: bool, location: SourceLocation) -> Result<(), StdError> {
    if cond {
        Ok(())
    } else {
        fail_with_message("test assertion failed: expected true, got false", location)
    }
}

/// `AssertFalse`: fail when `cond` is true.
pub(crate) fn assert_false(cond: bool, location: SourceLocation) -> Result<(), StdError> {
    if !cond {
        Ok(())
    } else {
        fail_with_message("test assertion failed: expected false, got true", location)
    }
}

/// `AssertEquals` for integer operands: `expected` first, `actual` second.
pub(crate) fn assert_equals_integer(
    expected: i64,
    actual: i64,
    location: SourceLocation,
) -> Result<(), StdError> {
    if expected == actual {
        Ok(())
    } else {
        fail_with_message(
            format!("test assertion failed: expected {expected}, got {actual}"),
            location,
        )
    }
}

/// `Fail`: unconditional test failure with user message.
pub(crate) fn fail(msg: String, location: SourceLocation) -> Result<(), StdError> {
    fail_with_message(format!("test failed: {msg}"), location)
}

/// `Skip`: record skip reason on stderr; does not fail the run.
pub(crate) fn skip(msg: String, _location: SourceLocation) -> Result<(), StdError> {
    eprintln!("test skipped: {msg}");
    Ok(())
}
