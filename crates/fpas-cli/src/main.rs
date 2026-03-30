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
mod cli_input;
mod cli_run;

use std::env;
use std::process;

pub(crate) use cli_input::{CliConfig, CliInput, resolve_cli_config, resolve_cli_input};
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

#[cfg(test)]
mod main_tests;

#[cfg(test)]
mod project;

#[cfg(test)]
mod test_support;
