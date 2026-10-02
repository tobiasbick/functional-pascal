//! Output contracts shared by CLI commands.

mod diagnostics;
mod failure;

use std::io::{self, Write};

pub(crate) use diagnostics::{DiagnosticFormat, Reporter, locate};
pub(crate) use failure::CliFailure;

/// Writes a command result to stdout and reports failures on stderr.
pub(crate) fn write_stdout(
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    description: &str,
    write: impl FnOnce(&mut dyn Write) -> io::Result<()>,
) -> Result<(), i32> {
    write(stdout).map_err(|error| report_write_error(stderr, description, &error))
}

/// Reports that a command could not complete its promised output.
pub(crate) fn report_write_error(
    stderr: &mut dyn Write,
    description: &str,
    error: &io::Error,
) -> i32 {
    let _ = writeln!(stderr, "{}", write_failure(description, error));
    1
}

/// Describes a failed write of promised command output.
pub(crate) fn write_failure(description: &str, error: &io::Error) -> CliFailure {
    CliFailure::new(
        fpas_diagnostics::codes::CLI_OUTPUT_FAILED,
        format!("Cannot write {description}: {error}"),
    )
}
