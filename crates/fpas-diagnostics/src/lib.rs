#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        clippy::unwrap_used,
        reason = "tests use explicit failures to keep fixture assertions focused"
    )
)]

//! Shared diagnostic data model and rendering utilities for the FPAS toolchain.

mod code;
mod diagnostic;
mod location;
mod render;
mod span;

/// Stable diagnostic code catalog used across all FPAS stages.
pub mod codes;

pub use code::{DiagnosticCode, InvalidDiagnosticCode};
pub use diagnostic::{Diagnostic, DiagnosticSeverity, DiagnosticStage};
pub use location::{SourceLocation, SourceLocationError};
pub use render::{render, render_without_path};
pub use span::{SourceSpan, SourceSpanError};
