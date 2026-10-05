//! Test-runner result banners and failure records in the selected diagnostic format.
//!
//! Text mode prints banners such as `  FAIL  name_test.fpas` with indented records
//! below them; JSON mode prints only the records.
//! **Documentation:** [`docs/pascal/tools/diagnostics.md`](../../../../docs/pascal/tools/diagnostics.md)

use fpas_diagnostics::{DiagnosticCode, FileDiagnostic};

use crate::cli_output::{CliFailure, Reporter};

const INDENT: &str = "        ";

/// Writes a result banner such as `PASS`, `FAIL`, `SKIP` or `TIMEOUT` in text mode.
pub(super) fn banner(reporter: &mut Reporter<'_>, label: &str, display: &str) {
    reporter.progress(format_args!("  {label}  {display}"));
}

/// Writes a diagnostic record below the current banner.
pub(super) fn record(reporter: &mut Reporter<'_>, record: &FileDiagnostic) {
    reporter.record_indented(record, INDENT);
}

/// Writes every record of a failure below the current banner.
pub(super) fn failure(reporter: &mut Reporter<'_>, failure: &CliFailure) {
    for item in failure.records() {
        record(reporter, item);
    }
}

/// Writes a runner message, splitting an optional `\n  help: ` hint into the record.
pub(super) fn message(reporter: &mut Reporter<'_>, code: DiagnosticCode, text: &str) {
    failure(reporter, &CliFailure::from_message(code, text));
}

/// Writes captured program standard output below a result line in text mode.
pub(super) fn captured_stdout(reporter: &mut Reporter<'_>, lines: &[String]) {
    if lines.is_empty() {
        return;
    }
    reporter.progress(format_args!("{INDENT}stdout:"));
    for line in lines {
        reporter.progress(format_args!("{INDENT}  {line}"));
    }
}
