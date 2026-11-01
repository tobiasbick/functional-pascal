//! Isolated shared-core analysis; no bytecode, execution or source publication.

use super::{protocol::*, report};
use crate::{
    cli_input::{QueryCliConfig, QueryKind, QuerySelector},
    oneshot::Files,
};
use fpas_language_service::{
    CancellationToken, SourceQueryError, SourceQueryOperation, SourceSession, TargetReference,
};
use std::{io::Read, path::Path};

pub(super) const ARGUMENT: &str = "__fpas-source-query";

pub(crate) fn worker(args: &[String]) -> Option<i32> {
    if args.first().map(String::as_str) != Some(ARGUMENT) {
        return None;
    }
    let Some(root) = args.get(1) else {
        return Some(1);
    };
    Some(if execute(&Files::attach(Path::new(root))).is_ok() {
        0
    } else {
        1
    })
}

fn execute(files: &Files) -> std::io::Result<()> {
    let request = files.read_request()?;
    let mut report = report::empty(&request.args, &request.cwd);
    let config = match crate::resolve_cli_config(&request.args, &request.cwd) {
        Ok(config @ (crate::ResolvedCli::Query(_) | crate::ResolvedCli::Refactor(_))) => config,
        _ => {
            return Err(std::io::Error::other(
                "Invalid private source-query request.",
            ));
        }
    };
    let cancel = files.cancel();
    let cancellation = CancellationToken::with_probe(move || cancel.is_file());
    let prepared = match &config {
        crate::ResolvedCli::Query(config) => {
            prepare(config, &request.cwd, &cancellation, &mut report)
        }
        crate::ResolvedCli::Refactor(config) => {
            super::refactor::prepare(config, &request.cwd, &cancellation, &mut report)
        }
        _ => return Err(std::io::Error::other("Invalid source-tool request.")),
    };
    if let Err(error) = prepared {
        report.diagnostics.extend(
            error
                .diagnostics
                .iter()
                .map(|d| report::fp_diagnostic(d, &request.cwd)),
        );
        report::diagnostic(&mut report, error.code, &error.message);
        report.outcome = Outcome::Error;
        report.exit_code = 1;
        if [
            "analysis_unavailable",
            "analysis_incomplete",
            "query_cancelled",
            "source_limit",
            "source_changed",
        ]
        .contains(&error.code)
        {
            report::incomplete(&mut report, error.code);
        }
    }
    files.publish(&report::finish(&mut report))
}

fn prepare(
    config: &QueryCliConfig,
    cwd: &Path,
    cancellation: &CancellationToken,
    report: &mut Report,
) -> Result<(), SourceQueryError> {
    report.request.selector = Some(match &config.selector {
        QuerySelector::Search(text) => Selector::Search { text: text.clone() },
        QuerySelector::Symbol(name) => Selector::Symbol { name: name.clone() },
        QuerySelector::Reference(path) => Selector::Reference {
            path: report::path(path, cwd),
        },
        QuerySelector::Position { path, line, column } => Selector::Position {
            path: report::path(path, cwd),
            line: *line,
            column: *column,
        },
    });
    report.request.options.insert(
        "timeout_ms".into(),
        serde_json::json!(config.timeout.saturating_mul(1000)),
    );
    report
        .request
        .options
        .insert("limit".into(), serde_json::json!(config.limit));
    let reference = match &config.selector {
        QuerySelector::Reference(path) => Some(read_reference(path)?),
        _ => None,
    };
    let recorded_library = reference
        .as_ref()
        .and_then(|r| r.context.standard_library.as_ref())
        .map(std::path::PathBuf::from);
    let library = if config.standard_library.is_some() {
        config.standard_library.clone()
    } else if reference.is_some() {
        recorded_library
    } else {
        crate::standard_library::default_standard_library_root()
            .map_err(|e| failure("standard_library", e))?
    };
    let input = config
        .project
        .as_deref()
        .or_else(|| reference.as_ref().map(|r| Path::new(&r.context.input)))
        .or(match &config.selector {
            QuerySelector::Position { path, .. } => Some(path.as_path()),
            _ => None,
        })
        .ok_or_else(|| failure("invalid_request", "A manifest context is required."))?;
    let mut session = SourceSession::load(input, library.as_deref(), &[], cancellation)?;
    report.context = Some(Context {
        scope: session.context().clone(),
        path_base: report::path(cwd, Path::new("")),
    });
    report.snapshot = Some(session.snapshot().clone());
    if let QuerySelector::Search(text) = &config.selector {
        let found = session.find_declarations(text, false, config.limit, cancellation)?;
        super::projection::find(report, found, cwd);
    } else {
        let operation = match config.kind {
            QueryKind::Refs => SourceQueryOperation::References,
            QueryKind::Definition => SourceQueryOperation::Definition,
            QueryKind::Type => SourceQueryOperation::Type,
            QueryKind::Inspect => SourceQueryOperation::Inspect,
            QueryKind::Callers => SourceQueryOperation::Callers,
            QueryKind::Callees => SourceQueryOperation::Callees,
            _ => {
                return Err(failure(
                    "invalid_request",
                    "This operation requires search text.",
                ));
            }
        };
        let (result, selected) = match &config.selector {
            QuerySelector::Position { path, line, column } => {
                let selected = session.service().snapshot(path)?;
                let offset = super::source_offset(&selected, *line, *column).ok_or_else(|| {
                    failure(
                        "invalid_position",
                        "The line or Unicode-scalar column is outside the source.",
                    )
                })?;
                (
                    session.query_position(path, offset, operation, config.limit, cancellation)?,
                    Some(selected),
                )
            }
            QuerySelector::Symbol(name) => {
                let result = session.query_symbol(name, operation, config.limit, cancellation)?;
                let selected = result.target.as_ref().and_then(|t| t.snapshot.clone());
                (result, selected)
            }
            QuerySelector::Reference(_) => {
                let reference = reference
                    .as_ref()
                    .ok_or_else(|| failure("invalid_request", "Missing target reference."))?;
                let result =
                    session.query_reference(reference, operation, config.limit, cancellation)?;
                let selected = result.target.as_ref().and_then(|t| t.snapshot.clone());
                (result, selected)
            }
            QuerySelector::Search(_) => {
                return Err(failure("invalid_request", "Unexpected search selector."));
            }
        };
        super::projection::query(report, result, selected.as_deref(), operation, cwd);
    }
    let _ = report::finish(report);
    if !session.verify().unwrap_or(false) {
        report::incomplete(report, "source_changed");
        report::diagnostic(
            report,
            "source_changed",
            "Sources, manifests or selected membership changed during the query.",
        );
    }
    Ok(())
}

pub(super) fn read_reference(path: &Path) -> Result<TargetReference, SourceQueryError> {
    let file = std::fs::File::open(path).map_err(|e| failure("reference_read", e.to_string()))?;
    let mut bytes = Vec::new();
    file.take(report::MAX_JSON as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| failure("reference_read", e.to_string()))?;
    if bytes.len() > report::MAX_JSON {
        return Err(failure(
            "invalid_request",
            "Target reference exceeds the public JSON limit.",
        ));
    }
    let reference: TargetReference = serde_json::from_slice(&bytes)
        .map_err(|e| failure("invalid_request", format!("Invalid target reference: {e}")))?;
    reference.validate()?;
    Ok(reference)
}

fn failure(code: &'static str, message: impl Into<String>) -> SourceQueryError {
    SourceQueryError {
        code,
        message: message.into(),
        diagnostics: Vec::new(),
    }
}
