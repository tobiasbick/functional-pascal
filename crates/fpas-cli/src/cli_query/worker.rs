//! Isolated saved-source discovery and analysis; never compiles or executes FPAS.

use super::report;
use crate::{
    cli_input::{QueryCliConfig, QueryKind},
    oneshot::Files,
};
use fpas_language_service::{CancellationToken, LanguageService, SourceQueryOperation};
use fpas_project::source_inputs::SourceInputs;
use serde_json::{Value, json};
use std::path::Path;

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
        Ok(crate::ResolvedCli::Query(config)) if config.kind != QueryKind::Rename => config,
        _ => {
            return Err(std::io::Error::other(
                "Invalid private source-query request.",
            ));
        }
    };
    let cancel = files.cancel();
    let cancellation = CancellationToken::with_probe(move || cancel.is_file());
    let probe = cancellation.clone();
    let inputs = SourceInputs::begin(move || probe.is_cancelled())?;
    if let Err((code, error, records)) = prepare(&config, &request.cwd, &cancellation, &mut report)
    {
        if let Some(items) = report["diagnostics"].as_array_mut() {
            items.extend(
                records
                    .iter()
                    .map(|record| report::fp_diagnostic(record, &request.cwd)),
            );
        }
        report::diagnostic(&mut report, code, &error);
    }
    let mut input_files = inputs.files().into_iter().map(|(path,bytes)| {
        let digest = ring::digest::digest(&ring::digest::SHA256,&bytes);
        let sha256 = digest.as_ref().iter().map(|b| format!("{b:02x}")).collect::<String>();
        json!({"path":report::path(&normalized(&path),&request.cwd),"sha256":sha256,"bytes":bytes.len(),
            "kind":if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("fpas")) {"source"} else {"manifest"}})
    }).collect::<Vec<_>>();
    input_files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    report["coverage"]["files"] = json!(input_files);
    if let Some(reason) = inputs.reason() {
        report::incomplete(&mut report, reason);
    }
    // Serialize before the final consistency check so text preparation shares the deadline.
    let _ = report::finish(&mut report);
    if !inputs.verify().unwrap_or(false) {
        report::incomplete(&mut report, "source_changed");
        report::diagnostic(
            &mut report,
            "source_changed",
            "Sources, manifests or selected file membership changed during the query.",
        );
    }
    files.publish(&report::finish(&mut report))
}

type Failure = (&'static str, String, Vec<fpas_diagnostics::FileDiagnostic>);

fn prepare(
    config: &QueryCliConfig,
    cwd: &Path,
    cancellation: &CancellationToken,
    report: &mut Value,
) -> Result<(), Failure> {
    for path in std::iter::once(&config.path).chain(config.project.iter()) {
        fpas_project::source_inputs::select(path)
            .map_err(|error| ("source_read", error.to_string(), Vec::new()))?;
    }
    let library = match &config.standard_library {
        Some(root) => Some(root.clone()),
        None => crate::standard_library::default_standard_library_root()
            .map_err(|error| ("standard_library", error, Vec::new()))?,
    };
    let input = config.project.as_deref().unwrap_or(&config.path);
    let mut service = match library {
        Some(root) => {
            LanguageService::load_with_standard_library_and_cancellation(input, &root, cancellation)
        }
        None => LanguageService::load_with_cancellation(input, cancellation),
    }
    .map_err(|e| ("context_error", e.to_string(), e.diagnostics()))?;
    let workspace = service.workspace();
    report["context"]["kind"] = json!(match workspace.kind() {
        fpas_language_service::WorkspaceKind::Loose => "loose",
        fpas_language_service::WorkspaceKind::Project => "project",
        fpas_language_service::WorkspaceKind::Workspace => "workspace",
        _ => "unavailable",
    });
    report["context"]["manifest"] = json!(workspace.manifest_path().map(|p| report::path(p, cwd)));
    let mut projects = workspace
        .projects()
        .iter()
        .map(|p| json!({"name":p.name(),"manifest":report::path(p.manifest_path(),cwd)}))
        .collect::<Vec<_>>();
    projects.sort_by(|a, b| a["manifest"].as_str().cmp(&b["manifest"].as_str()));
    report["context"]["projects"] = json!(projects);
    if !workspace.issues().is_empty() {
        return Err((
            "context_error",
            "The selected source context could not be loaded.".into(),
            workspace
                .issues()
                .iter()
                .flat_map(|i| i.diagnostics())
                .collect(),
        ));
    }
    if workspace
        .projects()
        .iter()
        .filter(|p| p.owns_source(&config.path))
        .count()
        > 1
    {
        return Err((
            "ambiguous_context",
            "Several projects directly own the selected source.".into(),
            Vec::new(),
        ));
    }
    let snapshot = service
        .snapshot(&config.path)
        .map_err(|e| ("source_read", e.to_string(), e.diagnostics()))?;
    if config.project.is_some()
        && service
            .workspace()
            .project_for_source(snapshot.path())
            .is_none()
    {
        return Err((
            "outside_context",
            "The source does not belong to the selected context.".into(),
            Vec::new(),
        ));
    }
    let offset = super::source_offset(&snapshot, config.line, config.column).ok_or_else(|| {
        (
            "invalid_position",
            "The line or Unicode-scalar column is outside the source.".into(),
            Vec::new(),
        )
    })?;
    report["request"]["file"] = json!(report::path(snapshot.path(), cwd));
    let operation = match config.kind {
        QueryKind::Refs => SourceQueryOperation::References,
        QueryKind::Definition => SourceQueryOperation::Definition,
        QueryKind::Type => SourceQueryOperation::Type,
        QueryKind::Rename => {
            return Err((
                "invalid_request",
                "Rename is not a source query worker operation.".into(),
                Vec::new(),
            ));
        }
    };
    let result = service
        .source_query(
            snapshot.path(),
            offset,
            operation,
            config.limit,
            cancellation,
        )
        .map_err(|e| {
            report::incomplete(report, "analysis_unavailable");
            ("analysis_unavailable", e.to_string(), e.diagnostics())
        })?;
    report["diagnostics"] = json!(
        result
            .diagnostics
            .iter()
            .map(|d| report::fp_diagnostic(d, cwd))
            .collect::<Vec<_>>()
    );
    let target = result.target.as_ref().map(|target| {
        let declaration = target.snapshot.as_ref().zip(target.symbol.as_ref())
            .and_then(|(s,t)| report::location(s,t.selection_span,cwd));
        json!({"name":target.name,"qualified_name":target.qualified_name,
            "kind":target.symbol.as_ref().map_or("builtin".into(),|s| format!("{:?}",s.kind).to_ascii_lowercase()),
            "origin":if target.snapshot.is_some() {"source"} else {"builtin"},"declaration":declaration})
    });
    report["target"] = json!(target);
    let mut matches = result
        .matches
        .iter()
        .filter_map(|m| report::location(&m.snapshot, m.span, cwd))
        .collect::<Vec<_>>();
    matches.sort_by(|a, b| {
        a["path"].as_str().cmp(&b["path"].as_str()).then_with(|| {
            a["byte_span"]["start"]
                .as_u64()
                .cmp(&b["byte_span"]["start"].as_u64())
        })
    });
    report["matches"] = json!(matches);
    if let Some(ty) = &result.expression_type {
        report["type"] = json!({"display":ty.type_name,"kind":ty.kind, "location":report::location(&snapshot,ty.span,cwd),"declaration":ty.declaration.as_ref().and_then(|d|report::location(&d.snapshot,d.span,cwd))});
    }
    let found = if operation == SourceQueryOperation::Type {
        result.expression_type.is_some()
    } else if operation == SourceQueryOperation::Definition {
        !result.matches.is_empty()
    } else {
        result.target.is_some()
    };
    report["outcome"] = json!(if found { "ok" } else { "not_found" });
    report["exit_code"] = json!(if found { 0 } else { 1 });
    report["coverage"] = json!({"complete":true,"reasons":[],"files":[],"returned_count":matches.len(),"total_count":matches.len()});
    let invalid_location = matches.len() != result.matches.len()
        || result
            .target
            .as_ref()
            .is_some_and(|t| t.snapshot.is_some() && report["target"]["declaration"].is_null())
        || result.expression_type.is_some() && report["type"]["location"].is_null();
    if !found {
        let reason = if result.target.as_ref().is_some_and(|t| t.snapshot.is_none()) {
            "no_source_definition"
        } else {
            "no_target"
        };
        report::diagnostic(
            report,
            reason,
            "No source construct satisfying this operation was selected.",
        );
    }
    for reason in result.reasons {
        report::incomplete(report, reason);
    }
    if invalid_location {
        report::incomplete(report, "invalid_location");
    }
    Ok(())
}

fn normalized(path: &Path) -> std::path::PathBuf {
    let text = path.to_string_lossy();
    if let Some(path) = text.strip_prefix("\\\\?\\UNC\\") {
        return format!("\\\\{path}").into();
    }
    text.strip_prefix("\\\\?\\")
        .map_or_else(|| path.into(), Into::into)
}
