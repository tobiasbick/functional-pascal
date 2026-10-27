//! Standard-library staging through the already-built compiler.
use crate::cli_input::StdlibCliConfig;
use crate::cli_output::{CliFailure, Reporter};
use fpas_diagnostics::codes::CLI_OUTPUT_FAILED;
use std::io::Write;

pub(crate) fn stage(
    config: StdlibCliConfig,
    stdout: &mut dyn Write,
    reporter: &mut Reporter<'_>,
) -> i32 {
    let result = if config.dry_run {
        writeln!(
            stdout,
            "Would recompile standard-library units in `{}` and replace `{}`.",
            config.source.display(),
            config.output.display()
        )
    } else {
        let counters = match fpas_build::stage_standard_library(
            &config.source,
            &config.output,
            &fpas_build::BuildOptions::default(),
        ) {
            Ok(counters) => counters,
            Err(error) => return reporter.failure(&error.into()),
        };
        writeln!(
            stdout,
            "Compiled {} standard-library unit(s), reused {}.",
            counters.compiled, counters.sidecar_reused
        )
    };
    match result {
        Ok(()) => 0,
        Err(error) => reporter.failure(&CliFailure::new(
            CLI_OUTPUT_FAILED,
            format!("Cannot write standard-library staging result: {error}"),
        )),
    }
}
