//! Checked arithmetic and numeric text shared by FPAS compile-time and runtime consumers.
//!
//! **Documentation:** `docs/pascal/language/basics/operators.md` and
//! `docs/pascal/language/basics/number-literals.md`.

#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        reason = "numeric contract tests report explicit fixture failures"
    )
)]

mod error;
/// Checked signed 64-bit arithmetic.
pub mod integer;
/// Finite binary64 arithmetic and conversions.
pub mod real;
/// Shared numeric literal bodies, input text, and lossless real formatting.
pub mod text;

pub use error::NumericError;

#[cfg(test)]
mod tests;
