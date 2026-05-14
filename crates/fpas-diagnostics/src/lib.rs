//! Shared diagnostic data model and rendering utilities for the FPAS toolchain.

mod code;
mod diagnostic;
mod location;

/// Stable diagnostic code catalog used across all FPAS stages.
pub mod codes;

pub use code::DiagnosticCode;
pub use diagnostic::{Diagnostic, DiagnosticSeverity, DiagnosticStage, render};
pub use location::{SourceLocation, SourceSpan};
