//! Debugger protocol dispatch and shared verified launch preparation.

mod failure;
mod target;
use crate::{DebugCliConfig, DebugProtocol};
pub(crate) use failure::Failure;
use std::{
    fs::File,
    io::{self, Write},
};
pub(crate) use target::prepare_target;

/// Runs an explicitly selected long-lived debugger protocol.
pub(crate) fn debug_cli(
    config: DebugCliConfig,
    stdout: Box<dyn Write + Send>,
    stderr: &mut dyn Write,
) -> i32 {
    let target = match prepare(&config) {
        Ok(target) => target,
        Err(error) => return fail(stderr, error),
    };
    let result = match config.protocol {
        DebugProtocol::Jsonl => {
            let server = match fpas_debug::jsonl::JsonlServer::new(target) {
                Ok(server) => server,
                Err(error) => return fail(stderr, error.to_string()),
            };
            match config.commands {
                Some(path) => File::open(path)
                    .and_then(|reader| fpas_debug::jsonl::serve_script(reader, stdout, server)),
                None => fpas_debug::jsonl::serve(io::stdin(), stdout, server),
            }
        }
        DebugProtocol::Dap => {
            let server = match fpas_debug::dap::DapServer::new(target) {
                Ok(server) => server,
                Err(error) => return fail(stderr, error.to_string()),
            };
            fpas_debug::dap::serve(io::stdin(), stdout, server)
        }
        DebugProtocol::Batch => return fail(stderr, "Batch requests use the isolated CLI host."),
    };
    match result {
        Ok(()) => 0,
        Err(error) => fail(stderr, format!("Debugger transport failed: {error}")),
    }
}

/// Compiles/loads a verified target with the selected execution limits.
pub(crate) fn prepare(config: &DebugCliConfig) -> Result<fpas_debug::PreparedDebugTarget, Failure> {
    let library =
        crate::standard_library::resolve_standard_library(config.standard_library.as_deref())
            .map_err(Failure::from)?;
    Ok(
        prepare_target(config, library.as_ref())?.with_execution_limits(
            fpas_vm::DebugExecutionLimits {
                max_instructions: config.instruction_limit,
                timeout: config.timeout,
                max_output_bytes: config.output_limit,
                ..Default::default()
            },
        ),
    )
}

fn fail(stderr: &mut dyn Write, message: impl std::fmt::Display) -> i32 {
    let _ = writeln!(stderr, "{message}");
    1
}
