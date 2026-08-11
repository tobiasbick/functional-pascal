//! Stable text and JSON output for scaffold plans.

use std::io::Write;

use serde::Serialize;

use crate::cli_input::InitReportFormat;

use super::plan::{ScaffoldPlan, display_path};
use super::write::WriteStatus;

#[derive(Serialize)]
struct InitReport {
    status: &'static str,
    kind: &'static str,
    name: String,
    root: String,
    manifest: String,
    files: Vec<String>,
}

/// Writes exactly one report to stdout and keeps output failures actionable.
pub(super) fn write(
    plan: &ScaffoldPlan,
    status: WriteStatus,
    format: Option<InitReportFormat>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let report = build_report(plan, status);
    let output = match format {
        Some(InitReportFormat::Json) => match serde_json::to_string_pretty(&report) {
            Ok(json) => format!("{json}\n"),
            Err(error) => {
                let _ = writeln!(stderr, "Cannot serialize init report: {error}");
                return 2;
            }
        },
        None => text_report(&report),
    };
    match crate::cli_output::write_stdout(stdout, stderr, "init report to stdout", |stdout| {
        stdout.write_all(output.as_bytes())
    }) {
        Ok(()) => 0,
        Err(exit_code) => exit_code,
    }
}

fn build_report(plan: &ScaffoldPlan, status: WriteStatus) -> InitReport {
    let manifest_path = plan.root.join(&plan.manifest);
    InitReport {
        status: status.as_str(),
        kind: plan.kind.as_str(),
        name: plan.name.clone(),
        root: display_path(&plan.root, &plan.cwd),
        manifest: display_path(&manifest_path, &plan.cwd),
        files: plan
            .files
            .iter()
            .map(|file| display_path(&file.path, &plan.cwd))
            .collect(),
    }
}

fn text_report(report: &InitReport) -> String {
    let mut output = format!(
        "status: {}\nkind: {}\nname: {}\nroot: {}\nmanifest: {}\nfiles:\n",
        report.status, report.kind, report.name, report.root, report.manifest
    );
    for file in &report.files {
        output.push_str("  - ");
        output.push_str(file);
        output.push('\n');
    }
    output
}
