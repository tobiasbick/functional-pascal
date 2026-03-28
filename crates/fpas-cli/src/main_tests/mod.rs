use super::{CliInput, render_cli_diagnostic, resolve_cli_input, run_cli, run_source};
use crate::test_support::{create_temp_dir, write_file, write_text};
use fpas_diagnostics::codes::COMPILE_INTRINSIC_ARITY_MISMATCH;
use fpas_diagnostics::{Diagnostic, DiagnosticStage, SourceSpan};
use std::fs;
use std::path::Path;

mod concurrency;
mod diagnostics;
mod input;
mod projects;
mod stdlib;
mod support;
mod visibility;
