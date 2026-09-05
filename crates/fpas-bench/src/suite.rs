//! Suite definition loading for FPAS programs and native tooling workloads.

mod command;
mod executable;
mod runner;

use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub use executable::ensure_release_fpas;
pub use runner::run_suite;

/// One curated benchmark from `docs/bench/suite.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct BenchSpec {
    /// Short identifier used in tables and JSON results.
    pub id: String,
    /// Filter group configured by the suite.
    pub group: String,
    /// Workload driver; omitted entries run an FPAS program.
    #[serde(default)]
    pub driver: BenchDriver,
    /// Path to the `.fpas` program or `.fpasprj` project, relative to the repository root.
    #[serde(default)]
    pub path: String,
    /// Arguments passed to the selected workload driver.
    pub args: Vec<String>,
    /// Maximum wall-clock runtime before the process is terminated.
    pub timeout_ms: u64,
}

/// Supported benchmark processes.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BenchDriver {
    /// Run an FPAS program through the release CLI.
    #[default]
    Fpas,
    /// Measure repeated editor analysis using fresh query snapshots.
    LanguageService,
    /// Measure project, standard-library, and disk-backed editor queries.
    LanguageServiceProject,
    /// Measure semantic analysis and IR lowering of branch-heavy programs.
    CompilerLowering,
    /// Measure complete CLI project builds and artifact reuse.
    ProjectBuild,
    /// Decode and validate unit objects containing shared debugger subtypes.
    UnitArtifact,
    /// Decode and admit complete program images with shared debugger subtypes.
    ProgramArtifact,
}

#[derive(Debug, Deserialize)]
struct SuiteFile {
    bench: Vec<BenchSpec>,
}

/// Outcome of a single benchmark process.
#[derive(Debug, Clone)]
pub struct BenchRun {
    /// Spec id.
    pub id: String,
    /// Parsed `elapsed: N ms` value.
    pub elapsed_ms: u64,
    /// Optional `throughput: …` line from stdout.
    pub throughput: Option<String>,
    /// Full captured stdout.
    pub raw_stdout: String,
}

/// Load `docs/bench/suite.toml` from the repository root.
pub fn load_suite(repo_root: &Path) -> Result<Vec<BenchSpec>, String> {
    let path = repo_root.join("docs/bench/suite.toml");
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    let file: SuiteFile = toml::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;
    if file.bench.is_empty() {
        return Err(format!("{} contains no [[bench]] entries", path.display()));
    }
    validate_specs(&path, &file.bench)?;
    Ok(file.bench)
}

fn validate_specs(path: &Path, specs: &[BenchSpec]) -> Result<(), String> {
    if let Some(spec) = specs
        .iter()
        .find(|spec| matches!(spec.driver, BenchDriver::Fpas) && spec.path.is_empty())
    {
        return Err(format!(
            "{} benchmark `{}` requires an FPAS source path",
            path.display(),
            spec.id
        ));
    }
    if let Some(spec) = specs.iter().find(|spec| spec.timeout_ms == 0) {
        return Err(format!(
            "{} benchmark `{}` has invalid timeout_ms 0",
            path.display(),
            spec.id
        ));
    }
    Ok(())
}

/// Return configured group names in first-appearance order.
pub fn group_names(specs: &[BenchSpec]) -> Vec<String> {
    let mut groups = Vec::new();
    for spec in specs {
        if !groups.contains(&spec.group) {
            groups.push(spec.group.clone());
        }
    }
    groups
}

/// Filter suite entries by optional group name.
pub fn filter_group(specs: &[BenchSpec], group: Option<&str>) -> Result<Vec<BenchSpec>, String> {
    let Some(group) = group else {
        return Ok(specs.to_vec());
    };
    let filtered: Vec<BenchSpec> = specs
        .iter()
        .filter(|spec| spec.group == group)
        .cloned()
        .collect();
    if filtered.is_empty() {
        let known_groups = group_names(specs).join(", ");
        return Err(format!(
            "no benchmarks in group `{group}` (known groups: {known_groups})"
        ));
    }
    Ok(filtered)
}

/// Unix timestamp seconds for result metadata.
pub fn unix_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{BenchSpec, filter_group, group_names, validate_specs};
    use std::path::Path;

    #[test]
    fn group_names_include_every_configured_group() {
        let specs = [spec("vm"), spec("concurrency"), spec("tui"), spec("vm")];
        assert_eq!(group_names(&specs), ["vm", "concurrency", "tui"]);
    }

    #[test]
    fn unknown_group_diagnostic_lists_every_configured_group() {
        let specs = [spec("vm"), spec("concurrency"), spec("tui")];
        assert_eq!(
            filter_group(&specs, Some("missing")).map(|_| ()),
            Err("no benchmarks in group `missing` (known groups: vm, concurrency, tui)".to_owned())
        );
    }

    #[test]
    fn fpas_driver_requires_a_source_path() {
        let mut spec = spec("vm");
        spec.path.clear();
        assert!(validate_specs(Path::new("suite.toml"), &[spec]).is_err());
    }

    #[test]
    fn suite_rejects_zero_timeout() {
        let mut spec = spec("vm");
        spec.timeout_ms = 0;
        assert_eq!(
            validate_specs(Path::new("suite.toml"), &[spec]),
            Err("suite.toml benchmark `vm_bench` has invalid timeout_ms 0".to_owned())
        );
    }

    fn spec(group: &str) -> BenchSpec {
        BenchSpec {
            id: format!("{group}_bench"),
            group: group.to_owned(),
            driver: super::BenchDriver::Fpas,
            path: "unused.fpas".to_owned(),
            args: Vec::new(),
            timeout_ms: 1,
        }
    }
}
