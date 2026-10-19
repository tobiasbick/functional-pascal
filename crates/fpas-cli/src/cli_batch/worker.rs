//! Isolated target preparation and shared-engine batch execution.

use crate::oneshot::{Files, read_bounded};
use fpas_debug::batch::{Input, Options, diagnostic, empty_report};
use serde_json::json;
use std::{
    path::Path,
    time::{Duration, Instant},
};

pub(super) const ARGUMENT: &str = "__fpas-debug-batch";

/// Handles only the private worker entry point before ordinary CLI output is created.
pub(crate) fn worker(args: &[String]) -> Option<i32> {
    if args.first().map(String::as_str) != Some(ARGUMENT) {
        return None;
    }
    let Some(root) = args.get(1) else {
        return Some(1);
    };
    Some(match execute(&Files::attach(Path::new(root))) {
        Ok(()) => 0,
        Err(_) => 1,
    })
}

fn execute(files: &Files) -> std::io::Result<()> {
    let request = files.read_request()?;
    let mut report = match crate::resolve_cli_config(&request.args, &request.cwd) {
        Ok(crate::ResolvedCli::Debug(config)) if config.protocol == crate::DebugProtocol::Batch => {
            match prepare(&config) {
                Ok((target, options)) => {
                    fpas_debug::batch::run(target, options, Box::new(std::io::stdout()), || {
                        files.cancel().is_file()
                    })
                }
                Err(error) => error.report(),
            }
        }
        Ok(_) => error_report("invalid_options", "Worker requires a batch debug request."),
        Err(error) => crate::cli_debug::Failure::from(crate::cli_output::CliFailure::from_message(
            fpas_diagnostics::codes::CLI_ARGUMENTS_INVALID,
            &error,
        ))
        .report(),
    };
    super::trace::preserve_request(&mut report, &request.args);
    super::schedule::preserve_request(&mut report, &request.args);
    files.publish(&fpas_debug::batch::finish_report(&mut report))?;
    // Keep the owning process alive until the parent has copied the result and killed its tree.
    let deadline = Instant::now() + Duration::from_secs(31);
    while Instant::now() < deadline && !files.cancel().is_file() {
        std::thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

fn error_report(code: &str, message: &str) -> serde_json::Value {
    let mut report = empty_report("error", 1);
    report["diagnostics"] = json!([diagnostic(code, "setup", message)]);
    report
}

fn prepare(
    config: &crate::DebugCliConfig,
) -> Result<(fpas_debug::PreparedDebugTarget, Options), crate::cli_debug::Failure> {
    let mut options = Options {
        schedule_seed: config.batch.schedule_seed,
        breakpoints: config.batch.breakpoints.clone(),
        expressions: config.batch.expressions.clone(),
        trace: config.batch.trace.clone(),
        trace_limit: config.batch.trace_limit.unwrap_or(1000),
        task: config.batch.task,
        frame: config.batch.frame,
        input: Vec::new(),
    };
    if let Some(path) = &config.batch.script {
        let bytes = read_bounded(path, 1_048_576).map_err(|error| error.to_string())?;
        let text = std::str::from_utf8(&bytes).map_err(|error| error.to_string())?;
        let script = crate::test_script::parse_script_text(text, path)?;
        for event in script.events {
            options.input.push(match event {
                crate::test_script::ScriptEvent::Readln { line } => Input::Line(line),
                crate::test_script::ScriptEvent::ReadkeyChars { chars } => {
                    if options.schedule_seed.is_some() { return Err("Seeded scheduling accepts text-line script input only; remove key events or --schedule-seed.".to_string().into()); }
                    Input::Keys(chars)
                },
            });
        }
    }
    Ok((crate::cli_debug::prepare(config)?, options))
}
