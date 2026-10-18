//! Command-line entry point for building, checking, running, and testing FPAS projects.

#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::panic,
        clippy::unwrap_used,
        reason = "CLI tests use unwrap/expect/panic to keep fixture assertions compact"
    )
)]
mod artifact_filename;
mod cli_batch;
mod cli_build;
mod cli_check;
mod cli_debug;
mod cli_doc;
mod cli_environment;
mod cli_failure_dump;
mod cli_fmt;
mod cli_init;
mod cli_input;
mod cli_lsp;
mod cli_output;
mod cli_paths;
mod cli_query;
mod cli_run;
mod cli_test;
mod native_executable;
mod oneshot;
mod process_tree;
mod source_publication;
mod standard_library;
mod test_script;

use std::env;
use std::io::IsTerminal;
use std::process;

/// Process-wide allocator; FPAS strings, arrays, and records allocate heavily.
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(test)]
pub(crate) use cli_input::{CliConfig, TestCliConfig, TestReportFormat};
pub(crate) use cli_input::{
    CliInput, DebugCliConfig, DebugProtocol, ResolvedCli, resolve_cli_config,
};
pub(crate) use cli_run::run_cli;
#[cfg(test)]
pub(crate) use cli_run::{render_cli_diagnostic, run_source};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if let Some(exit_code) = cli_query::worker(&args) {
        process::exit(exit_code);
    }
    if let Some(exit_code) = cli_batch::worker(&args) {
        process::exit(exit_code);
    }
    if let Some(exit_code) = cli_test::run_process_worker(&args) {
        process::exit(exit_code);
    }
    let color = std::io::stderr().is_terminal()
        && std::env::var_os("NO_COLOR").is_none_or(|value| value.is_empty());
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(e) => {
            if cli_query::selected(&args) {
                use std::io::Write;
                if let Err(error) = std::io::stdout().write_all(&cli_query::cwd_failure(&args, &e))
                {
                    eprintln!("Cannot write query output: {error}");
                }
                process::exit(1);
            }
            if cli_batch::selected(&args) {
                use std::io::Write;
                let mut report = fpas_debug::batch::empty_report("error", 1);
                report["diagnostics"] = serde_json::json!([fpas_debug::batch::diagnostic(
                    "current_directory",
                    "setup",
                    &e.to_string()
                )]);
                let bytes = fpas_debug::batch::finish_report(&mut report);
                if let Err(error) = std::io::stdout().write_all(&bytes) {
                    eprintln!("Cannot write batch result: {error}");
                }
                process::exit(1);
            }
            let mut stderr = std::io::stderr().lock();
            let mut reporter = cli_output::Reporter::new(
                cli_output::DiagnosticFormat::requested(&args),
                &mut stderr,
            )
            .with_color(color);
            process::exit(reporter.failure(&cli_output::CliFailure::new(
                fpas_diagnostics::codes::PROJECT_DIRECTORY_READ_FAILED,
                format!("Error reading current directory: {e}"),
            )));
        }
    };

    cli_run::authorize_process_lifecycle();
    let stdout: Box<dyn std::io::Write + Send> = Box::new(std::io::stdout());
    // Unlocked: program-output events from VM tasks write to the same stream.
    let mut stderr = std::io::stderr();
    let exit_code = run_cli(&args, &cwd, stdout, &mut stderr, color);
    if exit_code != 0 {
        process::exit(exit_code);
    }
}

// `main_tests` exercises the `fpas` binary (CLI + full pipeline). `project` tests target
// `fpas_project` loading/linking through the crate's `project` re-exports.
#[cfg(test)]
mod main_tests;

#[cfg(test)]
#[cfg(test)]
mod project;
mod project_build;

#[cfg(test)]
mod test_support;
