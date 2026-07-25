//! Persist and compare benchmark run results.

use crate::suite::BenchRun;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Saved suite snapshot written under `.temp-data/bench/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchSnapshot {
    /// User-provided label (`before`, `after`, …).
    pub label: String,
    /// Unix timestamp when the snapshot was written.
    pub timestamp_unix: u64,
    /// Per-bench measurements.
    pub benches: Vec<BenchResult>,
}

/// One bench row inside a snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchResult {
    /// Spec id.
    pub id: String,
    /// Elapsed milliseconds from the program output.
    pub elapsed_ms: u64,
    /// Optional throughput line.
    pub throughput: Option<String>,
    /// Full stdout for debugging.
    pub raw_stdout: String,
}

impl From<&BenchRun> for BenchResult {
    fn from(run: &BenchRun) -> Self {
        Self {
            id: run.id.clone(),
            elapsed_ms: run.elapsed_ms,
            throughput: run.throughput.clone(),
            raw_stdout: run.raw_stdout.clone(),
        }
    }
}

/// Directory for saved JSON snapshots.
pub fn results_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".temp-data/bench")
}

/// Path for a labeled snapshot file.
pub fn snapshot_path(repo_root: &Path, label: &str) -> Result<PathBuf, String> {
    validate_label(label)?;
    Ok(results_dir(repo_root).join(format!("{label}.json")))
}

/// Write a snapshot JSON file for `label`.
pub fn save_snapshot(
    repo_root: &Path,
    label: &str,
    timestamp_unix: u64,
    runs: &[BenchRun],
) -> Result<PathBuf, String> {
    let path = snapshot_path(repo_root, label)?;
    let dir = results_dir(repo_root);
    fs::create_dir_all(&dir)
        .map_err(|error| format!("failed to create {}: {error}", dir.display()))?;

    let snapshot = BenchSnapshot {
        label: label.to_owned(),
        timestamp_unix,
        benches: runs.iter().map(BenchResult::from).collect(),
    };
    let text = serde_json::to_string_pretty(&snapshot)
        .map_err(|error| format!("failed to serialize snapshot: {error}"))?;
    fs::write(&path, text)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    Ok(path)
}

/// Load a previously saved snapshot.
pub fn load_snapshot(repo_root: &Path, label: &str) -> Result<BenchSnapshot, String> {
    let path = snapshot_path(repo_root, label)?;
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn validate_label(label: &str) -> Result<(), String> {
    if label.is_empty() {
        return Err("label must not be empty".to_owned());
    }
    if !label
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err("label must contain only ASCII letters, digits, '-' and '_'".to_owned());
    }
    Ok(())
}

/// Comparison row for console output.
#[derive(Debug, Clone)]
pub struct CompareRow {
    /// Bench id.
    pub id: String,
    /// Baseline elapsed ms, if present.
    pub baseline_ms: Option<u64>,
    /// Current elapsed ms.
    pub current_ms: u64,
    /// Percent change vs baseline: `(current - baseline) / baseline * 100`.
    pub delta_pct: Option<f64>,
    /// Optional throughput from the current run.
    pub throughput: Option<String>,
}

/// Build comparison rows aligned by bench id.
pub fn compare_runs(baseline: &BenchSnapshot, current: &[BenchRun]) -> Vec<CompareRow> {
    current
        .iter()
        .map(|run| {
            let baseline_ms = baseline
                .benches
                .iter()
                .find(|entry| entry.id == run.id)
                .map(|entry| entry.elapsed_ms);
            let delta_pct = baseline_ms.map(|base| {
                if base == 0 {
                    0.0
                } else {
                    ((run.elapsed_ms as f64) - (base as f64)) / (base as f64) * 100.0
                }
            });
            CompareRow {
                id: run.id.clone(),
                baseline_ms,
                current_ms: run.elapsed_ms,
                delta_pct,
                throughput: run.throughput.clone(),
            }
        })
        .collect()
}

/// True when any compared bench slowed by more than `threshold_pct`.
pub fn has_regression(rows: &[CompareRow], threshold_pct: f64) -> bool {
    rows.iter().any(|row| match row.delta_pct {
        Some(delta) => delta > threshold_pct,
        None => false,
    })
}
