use super::{
    CliConfig, CliInput, ResolvedCli, render_cli_diagnostic, resolve_cli_config, run_cli,
    run_source,
};
use crate::cli_input::resolve_cli_input;
use crate::test_support::{create_temp_dir, write_file, write_text};
use fpas_diagnostics::codes::COMPILE_INTRINSIC_ARITY_MISMATCH;
use fpas_diagnostics::{Diagnostic, SourceSpan};
use std::fs;
use std::path::Path;

mod debugger;
mod diagnostics;
mod examples;
mod fmt;
mod init;
mod input;
mod network;
mod network_hardening;
mod network_streaming;
mod network_tls;
mod output;
mod projects;
mod standard_library;
#[path = "../../stdlib_sync.rs"]
mod stdlib_sync;
mod support;
mod test_project;
mod test_runner;
mod test_suite;
mod test_suite_negative;
mod tui;
#[path = "../../version_sync.rs"]
mod version_sync;
mod visibility;
