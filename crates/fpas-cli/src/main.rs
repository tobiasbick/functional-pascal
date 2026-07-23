#![cfg_attr(
    test,
    expect(
        clippy::expect_used,
        reason = "CLI tests use expect to keep command-path assertions compact"
    )
)]
#![cfg_attr(
    test,
    expect(
        clippy::panic,
        reason = "CLI tests use explicit panic for structural mismatches"
    )
)]
#![cfg_attr(
    test,
    expect(
        clippy::unwrap_used,
        reason = "CLI tests use unwrap to keep fixture path setup compact"
    )
)]
mod cli_check;
mod cli_fmt;
mod cli_input;
mod cli_paths;
mod cli_run;
mod cli_test;
mod standard_library;
mod test_script;

use std::env;
use std::process;

#[cfg(test)]
pub(crate) use cli_input::{CliConfig, TestCliConfig, TestReportFormat};
pub(crate) use cli_input::{CliInput, ResolvedCli, resolve_cli_config};
pub(crate) use cli_run::run_cli;
#[cfg(test)]
pub(crate) use cli_run::{render_cli_diagnostic, run_source};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(e) => {
            eprintln!("Error reading current directory: {e}");
            process::exit(1);
        }
    };

    let stdout: Box<dyn std::io::Write + Send> = Box::new(std::io::stdout());
    let mut stderr = std::io::stderr().lock();
    let exit_code = run_cli(&args, &cwd, stdout, &mut stderr);
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
