//! Suite definition loading and running FPAS benchmark programs.

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

/// One curated benchmark from `docs/bench/suite.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct BenchSpec {
    /// Short identifier used in tables and JSON results.
    pub id: String,
    /// Filter group (`vm` or `tui`).
    pub group: String,
    /// Path to the `.fpas` program, relative to the repository root.
    pub path: String,
    /// Arguments passed after `fpas run <path> --`.
    pub args: Vec<String>,
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
    Ok(file.bench)
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
        return Err(format!(
            "no benchmarks in group `{group}` (known groups: vm, tui)"
        ));
    }
    Ok(filtered)
}

/// Ensure the release `fpas` binary exists, building `fpas-cli` when missing.
pub fn ensure_release_fpas(repo_root: &Path) -> Result<PathBuf, String> {
    let fpas = release_fpas_path(repo_root);
    if fpas.is_file() {
        return Ok(fpas);
    }
    eprintln!("release fpas not found; building fpas-cli --release…");
    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "fpas-cli"])
        .current_dir(repo_root)
        .status()
        .map_err(|error| format!("failed to run cargo build: {error}"))?;
    if !status.success() {
        return Err(format!(
            "cargo build --release -p fpas-cli failed ({status})"
        ));
    }
    if !fpas.is_file() {
        return Err(format!(
            "expected release binary at {} after build",
            fpas.display()
        ));
    }
    Ok(fpas)
}

fn release_fpas_path(repo_root: &Path) -> PathBuf {
    let mut path = repo_root.join("target/release/fpas");
    if cfg!(windows) {
        path.set_extension("exe");
    }
    path
}

/// Run one suite entry and parse its elapsed time.
pub fn run_bench(repo_root: &Path, fpas: &Path, spec: &BenchSpec) -> Result<BenchRun, String> {
    let program = repo_root.join(&spec.path);
    if !program.is_file() {
        return Err(format!("benchmark source missing: {}", program.display()));
    }

    let mut command = Command::new(fpas);
    command
        .arg("run")
        .arg(&program)
        .arg("--")
        .args(&spec.args)
        .current_dir(repo_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = command
        .output()
        .map_err(|error| format!("failed to spawn {}: {error}", fpas.display()))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() {
        return Err(format!(
            "benchmark `{}` failed ({})\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}",
            spec.id, output.status
        ));
    }

    let elapsed_ms = parse_elapsed_ms(&stdout).ok_or_else(|| {
        format!(
            "benchmark `{}` stdout missing `elapsed: N ms` line:\n{stdout}",
            spec.id
        )
    })?;
    let throughput = parse_throughput_line(&stdout);

    Ok(BenchRun {
        id: spec.id.clone(),
        elapsed_ms,
        throughput,
        raw_stdout: stdout,
    })
}

/// Run all filtered specs in order.
pub fn run_suite(
    repo_root: &Path,
    fpas: &Path,
    specs: &[BenchSpec],
) -> Result<Vec<BenchRun>, String> {
    let mut runs = Vec::with_capacity(specs.len());
    for spec in specs {
        eprintln!("running {}…", spec.id);
        runs.push(run_bench(repo_root, fpas, spec)?);
    }
    Ok(runs)
}

/// Parse `elapsed: <int> ms` from benchmark stdout.
pub fn parse_elapsed_ms(stdout: &str) -> Option<u64> {
    for line in stdout.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("elapsed:") else {
            continue;
        };
        let rest = rest.trim();
        let Some(number) = rest.strip_suffix("ms") else {
            continue;
        };
        let number = number.trim();
        if let Ok(value) = number.parse::<u64>() {
            return Some(value);
        }
    }
    None
}

fn parse_throughput_line(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("throughput:"))
        .map(str::to_owned)
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
    use super::parse_elapsed_ms;

    #[test]
    fn parse_elapsed_ms_reads_standard_line() {
        let stdout = "iterations: 1\nelapsed: 142 ms\nthroughput: 7 ops/s\n";
        assert_eq!(parse_elapsed_ms(stdout), Some(142));
    }

    #[test]
    fn parse_elapsed_ms_ignores_noise() {
        assert_eq!(parse_elapsed_ms("no timing here\n"), None);
    }
}
