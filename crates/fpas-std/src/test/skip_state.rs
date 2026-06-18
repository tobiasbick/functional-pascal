//! Thread-local skip flag for `Std.Test.Skip` (read by the `fpas test` runner).
//!
//! **Documentation:** `docs/pascal/std/testing/test.md` (from the repository root).

use std::cell::Cell;

thread_local! {
    static TEST_SKIPPED: Cell<bool> = const { Cell::new(false) };
}

/// Clears the skip flag before a test program runs.
pub fn reset_test_skip_state() {
    TEST_SKIPPED.with(|flag| flag.set(false));
}

/// Returns whether `Skip` was called on the current thread since the last reset.
pub fn test_was_skipped() -> bool {
    TEST_SKIPPED.with(|flag| flag.get())
}

pub(crate) fn mark_test_skipped() {
    TEST_SKIPPED.with(|flag| flag.set(true));
}
