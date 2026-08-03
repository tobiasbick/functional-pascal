//! Output contracts shared by CLI commands.

use std::io::{self, Write};

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
    let _ = writeln!(stderr, "Cannot write {description}: {error}");
    1
}
