//! One bounded execution, one immutable snapshot, and no external session protocol.

mod diagnostics;
mod report;
pub use diagnostics::fp_diagnostic;
mod snapshot;
pub use report::{MAX_REPORT_BYTES, diagnostic, empty_report, finish_report};

use crate::{
    PreparedDebugTarget,
    engine::{
        DebugEngine, DebugEvent, DebugOp, DebugRecord, DebugRequest, DebugStatus, ResponseBody,
    },
    evaluation::parse_debug_expression,
};
use fpas_vm::{DebugExpression, SourceBreakpoint};
use serde_json::{Value, json};

/// Input queued before the single resume, sharing the runner's script semantics.
#[derive(Debug, Clone)]
pub enum Input {
    /// One line for ReadLn or line-buffered ReadText.
    Line(String),
    /// Characters for the headless terminal key queue.
    Keys(String),
}

/// Complete batch inspection request, independent of argv and transport.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Repeatable source locations, resolved before bytecode runs.
    pub breakpoints: Vec<String>,
    /// Ordered non-executing expressions.
    pub expressions: Vec<String>,
    /// Explicit task context, otherwise the stopping task.
    pub task: Option<u64>,
    /// Explicit frame depth, otherwise the active frame.
    pub frame: Option<usize>,
    /// Validated, bounded scripted program input.
    pub input: Vec<Input>,
}

/// Parse a path's one-based numeric suffixes without treating drive letters as positions.
pub fn breakpoint(text: &str) -> Result<SourceBreakpoint, String> {
    let (prefix, last) = text
        .rsplit_once(':')
        .ok_or("Use path:line or path:line:column.")?;
    let number = |text: &str| {
        text.parse::<u32>()
            .ok()
            .filter(|n| *n > 0)
            .ok_or_else(|| "Breakpoint coordinates must be positive integers.".to_string())
    };
    let last = number(last)?;
    let (source, line, column) = match prefix.rsplit_once(':') {
        Some((source, line)) if line.bytes().all(|byte| byte.is_ascii_digit()) => {
            (source, number(line)?, Some(last))
        }
        _ => (prefix, last, None),
    };
    if source.is_empty() {
        return Err("A breakpoint requires a source path.".into());
    }
    Ok(SourceBreakpoint {
        source: source.into(),
        line,
        column,
    })
}

/// Executes one prepared target through the shared engine and captures its first result.
/// Hosted output uses the supplied writer; isolated hosts share it with native stdout.
#[expect(
    clippy::unwrap_used,
    reason = "This function creates its own empty_report envelope with a fixed diagnostics array."
)]
pub fn run(
    target: PreparedDebugTarget,
    options: Options,
    output: Box<dyn std::io::Write + Send>,
    mut cancelled: impl FnMut() -> bool,
) -> Value {
    let limits = target.execution_limits();
    let mut report = empty_report("error", 1);
    report["limits"]["timeout_seconds"] = json!(limits.timeout.as_secs());
    report["limits"]["instruction_limit"] = json!(limits.max_instructions);
    report["limits"]["output_bytes"] = json!(limits.max_output_bytes);
    let expressions = match options
        .expressions
        .iter()
        .map(|text| {
            let expression =
                parse_debug_expression(text, fpas_vm::DebugEvaluationLimits::default())
                    .map_err(|error| error.message)?;
            if !fpas_vm::non_executing(&expression) {
                return Err("Batch --print accepts only non-executing expressions.".into());
            }
            Ok(expression)
        })
        .collect::<Result<Vec<DebugExpression>, String>>()
    {
        Ok(expressions) => expressions,
        Err(error) => {
            report["diagnostics"] = json!([diagnostic("expression_syntax", "request", &error)]);
            return report;
        }
    };
    let mut locations = Vec::new();
    for text in &options.breakpoints {
        let mut location = match breakpoint(text) {
            Ok(location) => location,
            Err(error) => {
                report["diagnostics"] =
                    json!([diagnostic("breakpoint_invalid", "request", &error)]);
                return report;
            }
        };
        let matching = target
            .sources()
            .iter()
            .filter(|source| {
                source.path == location.source
                    || source
                        .original_path
                        .as_ref()
                        .is_some_and(|path| path == std::path::Path::new(&location.source))
            })
            .collect::<Vec<_>>();
        if matching.len() == 1 {
            location.source = matching[0].path.clone();
        }
        if !locations.contains(&location) {
            locations.push(location);
        }
    }
    report["sources"] = json!(
        target
            .source_paths()
            .iter()
            .enumerate()
            .map(|(id, path)| json!({"id":id,"path":path}))
            .collect::<Vec<_>>()
    );
    let mut engine = match DebugEngine::new(target) {
        Ok(engine) => engine,
        Err(error) => {
            report["diagnostics"] = json!([diagnostic(
                "debugger_initialization",
                "setup",
                &error.message
            )]);
            return report;
        }
    };
    engine.retain_for_batch();
    if let Some(session) = engine.batch_session() {
        session.terminal_handle().set_output_writer(output);
    }
    let mut id = 1;
    let _ = engine.execute(DebugRequest::new(id, DebugOp::Initialize));
    let mut resolved = Vec::new();
    for location in locations {
        id += 1;
        let records = engine.execute(DebugRequest::new(
            id,
            DebugOp::BreakpointSet {
                source: location.source,
                line: location.line,
                column: location.column,
                assign: None,
                condition: None,
                hit_condition: None,
                log_message: None,
            },
        ));
        let bound = records.iter().find_map(|record| match record {
            DebugRecord::Response {
                outcome: Ok(ResponseBody::Breakpoint(bound)),
                ..
            } => Some(bound),
            _ => None,
        });
        if let Some(bound) = bound.filter(|bound| bound.instruction.is_some()) {
            resolved.push(json!({"source":bound.requested.source,"requested_line":bound.requested.line,"requested_column":bound.requested.column,"location":bound.location.as_ref().map(|location| json!({"source":location.source,"line":location.line,"column":location.column}))}));
        } else {
            let message = records
                .iter()
                .find_map(|record| match record {
                    DebugRecord::Response {
                        outcome: Err(error),
                        ..
                    } => Some(error.message.as_str()),
                    _ => None,
                })
                .unwrap_or("Breakpoint has no executable source position.");
            report["diagnostics"] = json!([diagnostic("breakpoint_unbound", "setup", message)]);
            return report;
        }
    }
    id += 1;
    let _ = engine.execute(DebugRequest::new(
        id,
        DebugOp::Launch {
            stop_on_entry: true,
        },
    ));
    if let Some(session) = engine.batch_session() {
        for input in options.input {
            let outcome = match input {
                Input::Line(line) => session.push_debuggee_input(&line).map(|_| ()),
                Input::Keys(chars) => session.push_debuggee_keys(&chars).map(|_| ()),
            };
            if let Err(error) = outcome {
                report["diagnostics"] =
                    json!([diagnostic("input_invalid", "setup", &error.message)]);
                return report;
            }
        }
        if let Err(error) = session.signal_debuggee_eof() {
            report["diagnostics"] = json!([diagnostic("input_invalid", "setup", &error.message)]);
            return report;
        }
    }
    id += 1;
    let initial = engine.execute(DebugRequest::new(id, DebugOp::Continue));
    let mut records = initial;
    let mut cancellation_requested = false;
    loop {
        for record in records.drain(..) {
            match record {
                // The host writer already captures console writes without line buffering.
                DebugRecord::Event(DebugEvent::Output { .. }) => {}
                DebugRecord::Event(DebugEvent::Stopped(stop)) => {
                    let runtime = stop.diagnostic.is_some();
                    report["outcome"] = json!(if runtime {
                        "runtime_error"
                    } else {
                        "breakpoint"
                    });
                    report["exit_code"] = json!(if runtime { 2 } else { 0 });
                    report["stop"] = json!({"reason":stop.reason.as_str(),"task_id":stop.task_id,"location":stop.location.as_ref().map(|loc| json!({"source":loc.source,"line":loc.line,"column":loc.column})),"breakpoints":resolved});
                    if let Some(error) = stop.diagnostic {
                        report["diagnostics"]
                            .as_array_mut()
                            .unwrap()
                            .push(fp_diagnostic(None, None, &error));
                    }
                }
                DebugRecord::Event(DebugEvent::Terminated {
                    reason: "completed",
                    ..
                }) => {
                    report["outcome"] = json!("completed");
                    report["exit_code"] = json!(if resolved.is_empty() { 0 } else { 3 });
                    if !resolved.is_empty() {
                        report["diagnostics"]
                            .as_array_mut()
                            .unwrap()
                            .push(diagnostic(
                                "breakpoint_not_hit",
                                "execution",
                                "Program completed without reaching a requested breakpoint.",
                            ));
                    }
                }
                DebugRecord::Event(DebugEvent::ProtocolError(error))
                | DebugRecord::Response {
                    outcome: Err(error),
                    ..
                } => {
                    report["outcome"] =
                        json!(
                            if ["instruction_limit", "output_limit", "execution_timeout"]
                                .contains(&error.code.as_str())
                            {
                                "limit"
                            } else {
                                "error"
                            }
                        );
                    report["exit_code"] = json!(3);
                    report["diagnostics"]
                        .as_array_mut()
                        .unwrap()
                        .push(diagnostic(&error.code, "execution", &error.message));
                }
                _ => {}
            }
        }
        if engine.status() != DebugStatus::Running {
            break;
        }
        if !cancellation_requested && cancelled() {
            cancellation_requested = true;
            id += 1;
            let _ = engine.execute(DebugRequest::new(id, DebugOp::Pause));
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
        records = engine.poll();
    }
    if let Some(session) = engine.batch_session() {
        let state = session.capture_snapshot(&expressions, options.task, options.frame);
        snapshot::append(&mut report, state, &options.expressions);
        session.disconnect();
    } else {
        report["capture"]["incomplete"] = json!(true);
        report["capture"]["reasons"] = json!(["state_unavailable"]);
    }
    if cancellation_requested {
        report["outcome"] = json!("limit");
        report["exit_code"] = json!(3);
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numeric_suffixes_preserve_windows_drives() {
        let b = breakpoint(r"C:\src\main.fpas:42:7").unwrap();
        assert_eq!(b.source, r"C:\src\main.fpas");
        assert_eq!((b.line, b.column), (42, Some(7)));
        assert_eq!(breakpoint(r"C:\src\main.fpas:42").unwrap().column, None);
        assert!(breakpoint("main.fpas:0").is_err());
    }
}
