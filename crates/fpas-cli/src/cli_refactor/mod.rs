//! Checked recipe transport and the parent-owned publication boundary.

mod apply;
mod artifact;
mod plan;
use super::{protocol::*, report};
use crate::cli_input::{RefactorAction, RefactorCliConfig};
pub(super) use apply::Publication;
use fpas_language_service::{CancellationToken, RefactorStatus, SourceQueryError, SourceSession};
use std::{path::Path, time::Instant};

pub(super) fn prepare(
    config: &RefactorCliConfig,
    cwd: &Path,
    cancellation: &CancellationToken,
    report: &mut Report,
) -> Result<(), SourceQueryError> {
    let saved = match &config.action {
        RefactorAction::Apply { plan } => Some(plan::read(plan)?),
        _ => None,
    };
    let reference = plan::reference(config)?;
    let context = saved
        .as_ref()
        .map(|p| &p.context)
        .or_else(|| reference.as_ref().map(|r| &r.context));
    let input = config
        .context
        .as_deref()
        .or_else(|| context.map(|c| Path::new(&c.input)))
        .or_else(|| plan::source(config))
        .ok_or_else(|| failure("invalid_request", "Missing manifest context."))?;
    let library = if let Some(library) = &config.standard_library {
        Some(library.clone())
    } else if let Some(context) = context {
        context
            .standard_library
            .as_ref()
            .map(std::path::PathBuf::from)
    } else {
        crate::standard_library::default_standard_library_root()
            .map_err(|e| failure("standard_library", e))?
    };
    let mut session = SourceSession::load(input, library.as_deref(), &[], cancellation)?;
    report.context = Some(Context {
        scope: session.context().clone(),
        path_base: report::path(cwd, Path::new("")),
    });
    report.snapshot = Some(session.snapshot().clone());
    let prepared = if let Some(plan) = &saved {
        session.revalidate_refactor(plan, cancellation)?
    } else {
        let recipe = plan::recipe(config, reference, &mut session)?;
        report.request.arguments.insert(
            "recipe".into(),
            serde_json::to_value(&recipe).map_err(|e| failure("invalid_request", e.to_string()))?,
        );
        session.prepare_refactor(&recipe, cancellation)?
    };
    report.request.options.insert(
        "timeout_ms".into(),
        serde_json::json!(config.timeout.saturating_mul(1000)),
    );
    report.coverage.reasons.clear();
    report.coverage.analysis_complete = prepared.status != RefactorStatus::Incomplete;
    report.diagnostics = prepared
        .diagnostics
        .iter()
        .map(|d| report::fp_diagnostic(d, cwd))
        .collect();
    report.outcome = match prepared.status {
        RefactorStatus::Incomplete => Outcome::Incomplete,
        RefactorStatus::Unsupported => Outcome::Error,
        _ => Outcome::Ok,
    };
    report.exit_code = match report.outcome {
        Outcome::Ok => 0,
        Outcome::Incomplete => 3,
        _ => 1,
    };
    if matches!(
        prepared.status,
        RefactorStatus::Unsupported | RefactorStatus::Incomplete
    ) {
        for blocker in &prepared.blockers {
            report::diagnostic(report, &blocker.code, &blocker.message);
        }
        report
            .coverage
            .reasons
            .extend(prepared.blockers.iter().map(|b| b.code.clone()));
    }
    report.result = Some(Payload::Refactor {
        status: prepared.status,
        plan: prepared.plan.map(Box::new),
        blockers: prepared.blockers,
        limitations: prepared.limitations,
        publication: None,
    });
    // The worker must reject an oversized applicable plan before artifact output/publication.
    let _ = report::finish(report);
    if !session.verify()? {
        return Err(failure(
            "source_changed",
            "Inputs changed during refactoring preparation.",
        ));
    }
    Ok(())
}

pub(super) fn coordinate(config: &RefactorCliConfig, report: &mut Report, deadline: Instant) {
    if report.outcome != Outcome::Ok {
        return;
    }
    let Some(Payload::Refactor {
        plan: Some(plan),
        status,
        ..
    }) = &report.result
    else {
        return;
    };
    let plan = plan.clone();
    let status = *status;
    let result = if matches!(config.action, RefactorAction::Apply { .. })
        && status == RefactorStatus::Prepared
    {
        apply::publish(&plan, deadline, report)
    } else if let Some(output) = &config.output {
        artifact::write(output, config, &plan, report, deadline)
    } else {
        Ok(())
    };
    if let Err(mut error) = result {
        if error.code == "query_cancelled" && Instant::now() >= deadline {
            error.code = "execution_timeout";
            error.message = "Preparation deadline expired before publication.".into();
        }
        report.outcome = Outcome::Error;
        report.exit_code = if ["publication_failed", "execution_timeout"].contains(&error.code) {
            3
        } else {
            1
        };
        report.coverage.reasons.insert(error.code.into());
        report::diagnostic(report, error.code, &error.message);
        if error.code == "execution_timeout" {
            report::incomplete(report, error.code);
        }
    }
}

pub(super) fn reload(
    plan: &fpas_language_service::RefactorPlan,
    deadline: Instant,
) -> Result<SourceSession, SourceQueryError> {
    if Instant::now() >= deadline {
        return Err(failure(
            "execution_timeout",
            "Preparation deadline expired before publication.",
        ));
    }
    let cancellation = CancellationToken::with_probe(move || Instant::now() >= deadline);
    let session = SourceSession::load(
        Path::new(&plan.context.input),
        plan.context.standard_library.as_deref().map(Path::new),
        &[],
        &cancellation,
    )?;
    if session.context() != &plan.context
        || session.snapshot().inventory != plan.before
        || !session.verify()?
    {
        return Err(failure(
            "stale_plan",
            "Inputs changed after isolated preparation; no source was published.",
        ));
    }
    Ok(session)
}

fn failure(code: &'static str, message: impl Into<String>) -> SourceQueryError {
    SourceQueryError {
        code,
        message: message.into(),
        diagnostics: Vec::new(),
    }
}
