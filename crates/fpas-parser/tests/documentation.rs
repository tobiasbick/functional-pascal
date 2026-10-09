//! Syntax regression checks for examples extracted from the current handbook.

#![expect(
    clippy::expect_used,
    reason = "handbook extraction fails fast when required sections or columns disappear"
)]

mod documentation {
    mod corrections;
    mod declarations;
    mod examples;
    mod generics;
    mod markdown;
    mod task_calls;
}
