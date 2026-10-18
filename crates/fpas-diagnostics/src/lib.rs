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
mod file_diagnostic;
mod fix;
mod foreign_habit;
mod location;
mod render;
mod source_range;
mod span;

/// Stable diagnostic code catalog used across all FPAS stages.
pub mod codes;

pub use code::{DiagnosticCode, InvalidDiagnosticCode};
pub use diagnostic::{Diagnostic, DiagnosticSeverity, DiagnosticStage};
pub use file_diagnostic::FileDiagnostic;
pub use fix::{Applicability, Fix, FixEdit, apply_edits, apply_machine_applicable, fix_title};
pub use foreign_habit::ForeignHabit;
pub use location::{SourceLocation, SourceLocationError};
pub use render::{
    TextStyle, render, render_json, render_program_stderr_json, render_text, render_without_path,
};
pub use source_range::{SourcePosition, SourceRange};
pub use span::{SourceSpan, SourceSpanError};
