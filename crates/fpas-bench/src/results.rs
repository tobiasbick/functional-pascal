//! Persist and compare benchmark run results.

use crate::suite::BenchRun;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

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

/// Path of the committed Markdown history file.
pub fn history_path(repo_root: &Path) -> PathBuf {
    repo_root.join("docs/bench/history.md")
}

const HISTORY_HEADER: &str = "\
# Benchmark history

Committed snapshots from `cargo bench-fpas record`. Absolute times are machine-specific; use them
to track relative progress on the same machine and to see which changes moved which benches.

Update after a meaningful performance change:

```sh
cargo bench-fpas record \"short note about the change\"
cargo bench-fpas record \"vm-only note\" --group vm
```

Newest entries are prepended below this header.
";

/// Append (prepend after header) a dated Markdown entry to [`history_path`].
pub fn record_history(
    repo_root: &Path,
    title: &str,
    group: Option<&str>,
    runs: &[BenchRun],
) -> Result<PathBuf, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("record title must not be empty".to_owned());
    }
    if title.contains('\n') || title.contains('\r') {
        return Err("record title must be a single line".to_owned());
    }

    let path = history_path(repo_root);
    let existing = if path.is_file() {
        fs::read_to_string(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?
    } else {
        HISTORY_HEADER.to_owned()
    };

    let entry = format_history_entry(title, group, runs);
    let updated = insert_history_entry(&existing, &entry);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    fs::write(&path, updated)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    Ok(path)
}

fn format_history_entry(title: &str, group: Option<&str>, runs: &[BenchRun]) -> String {
    let date = today_iso_date();
    let host = hostname_label();
    let group_label = group.unwrap_or("all");
    let mut out = String::new();
    out.push_str(&format!("## {date} — {title}\n\n"));
    out.push_str(&format!("- Host: `{host}`\n"));
    out.push_str(&format!("- Group: `{group_label}`\n"));
    out.push_str("- Suite: [`suite.toml`](suite.toml)\n\n");
    out.push_str("| bench | elapsed_ms | throughput |\n");
    out.push_str("|-------|------------|------------|\n");
    for run in runs {
        let throughput = run.throughput.as_deref().unwrap_or("-").replace('|', "\\|");
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            run.id, run.elapsed_ms, throughput
        ));
    }
    out.push('\n');
    out
}

fn insert_history_entry(existing: &str, entry: &str) -> String {
    const MARKER: &str = "Newest entries are prepended below this header.";
    if let Some(idx) = existing.find(MARKER) {
        let after_marker = idx + MARKER.len();
        let (head, tail) = existing.split_at(after_marker);
        let tail = tail.trim_start_matches(['\r', '\n']);
        let mut out = String::with_capacity(existing.len() + entry.len() + 2);
        out.push_str(head);
        out.push_str("\n\n");
        out.push_str(entry);
        if !tail.is_empty() {
            out.push_str(tail);
            if !tail.ends_with('\n') {
                out.push('\n');
            }
        }
        return out;
    }

    let mut out = String::from(HISTORY_HEADER);
    out.push('\n');
    out.push_str(entry);
    if !existing.trim().is_empty() && existing.trim() != HISTORY_HEADER.trim() {
        out.push_str(existing.trim_start());
        if !existing.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

fn today_iso_date() -> String {
    // Local calendar date via UTC day is good enough for history labels.
    let secs = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86_400;
    let (year, month, day) = civil_from_days(days as i64);
    format!("{year:04}-{month:02}-{day:02}")
}

/// Howard Hinnant civil-from-days (UTC), sufficient for dated history headings.
fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m as u32, d as u32)
}

fn hostname_label() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".to_owned())
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

#[cfg(test)]
mod tests {
    use super::{HISTORY_HEADER, format_history_entry, insert_history_entry};
    use crate::suite::BenchRun;

    #[test]
    fn insert_history_entry_prepends_after_header_marker() {
        let existing = format!("{HISTORY_HEADER}\n## old\n\n");
        let entry = "## new\n\n";
        let updated = insert_history_entry(&existing, entry);
        let new_pos = updated.find("## new").expect("new entry");
        let old_pos = updated.find("## old").expect("old entry");
        assert!(new_pos < old_pos);
        assert!(updated.contains("Newest entries are prepended below this header."));
    }

    #[test]
    fn format_history_entry_builds_markdown_table() {
        let runs = vec![BenchRun {
            id: "string_length".to_owned(),
            elapsed_ms: 92,
            throughput: Some("throughput: 5434782 lengths/s".to_owned()),
            raw_stdout: String::new(),
        }];
        let text = format_history_entry("after char_len cache", Some("vm"), &runs);
        assert!(text.contains("## "));
        assert!(text.contains("after char_len cache"));
        assert!(text.contains("| string_length | 92 |"));
        assert!(text.contains("Group: `vm`"));
    }
}
