//! JSON Lines routing of program stderr beside diagnostic records.
//!
//! Documentation: `docs/pascal/tools/diagnostics.md`.

use std::io::Write;
use std::sync::Arc;

use super::DiagnosticFormat;

/// Wraps child-process stderr as JSON records on process stderr; text mode inherits stderr.
pub(crate) fn program_stderr_receiver(format: DiagnosticFormat) -> Option<fpas_std::ProgramStderr> {
    program_stderr_receiver_with(format, |bytes| {
        let _ = std::io::stderr().write_all(bytes);
    })
}

/// Sends complete JSON program-output lines to the supplied sink in JSON mode.
pub(crate) fn program_stderr_receiver_with(
    format: DiagnosticFormat,
    write: impl Fn(&[u8]) + Send + Sync + 'static,
) -> Option<fpas_std::ProgramStderr> {
    (format == DiagnosticFormat::Json).then(|| {
        Arc::new(move |text: &str| {
            if let Ok(line) = fpas_diagnostics::render_program_stderr_json(text) {
                write(format!("{line}\n").as_bytes());
            }
        }) as fpas_std::ProgramStderr
    })
}
