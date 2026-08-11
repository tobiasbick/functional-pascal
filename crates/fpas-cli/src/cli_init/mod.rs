//! Non-interactive project, library, and workspace scaffolding.
//!
//! Documentation: [Initializing projects and workspaces](../../../../docs/pascal/program-structure/initializing.md).

pub(crate) mod naming;
mod plan;
mod report;
mod templates;
mod write;

use std::io::Write;

use crate::cli_input::InitCliConfig;

/// Creates or previews the scaffold selected by `fpas init`.
pub(crate) fn init_cli(
    config: InitCliConfig,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let scaffold = plan::build(&config);
    let status = if config.dry_run {
        write::WriteStatus::Planned
    } else {
        match write::apply(&scaffold) {
            Ok(status) => status,
            Err(message) => {
                let _ = writeln!(stderr, "{message}");
                return 1;
            }
        }
    };

    report::write(&scaffold, status, config.report, stdout, stderr)
}
