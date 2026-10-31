//! Persist and compare benchmark run results.

mod comparison;
mod publication;

use crate::suite::BenchRun;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub use comparison::{CompareRow, ComparisonBaseline, has_regression};
use publication::write_text;

/// Saved suite snapshot written under `.temp-data/bench/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchSnapshot {
    /// User-provided label (`before`, `after`, …).
    pub label: String,
    /// Unix timestamp when the snapshot was written.
    pub timestamp_unix: u64,
    /// Selected suite group, or `None` for the complete suite.
    pub group: Option<String>,
    /// Per-bench measurements.
    pub benches: Vec<BenchRun>,
}

impl BenchSnapshot {
    fn from_runs(label: &str, timestamp_unix: u64, group: Option<&str>, runs: &[BenchRun]) -> Self {
        Self {
            label: label.to_owned(),
            timestamp_unix,
            group: group.map(str::to_owned),
            benches: runs.to_vec(),
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
    group: Option<&str>,
    runs: &[BenchRun],
) -> Result<PathBuf, String> {
    let path = snapshot_path(repo_root, label)?;
    let dir = results_dir(repo_root);
    fs::create_dir_all(&dir)
        .map_err(|error| format!("failed to create {}: {error}", dir.display()))?;

    let snapshot = BenchSnapshot::from_runs(label, timestamp_unix, group, runs);
    let text = serde_json::to_string_pretty(&snapshot)
        .map_err(|error| format!("failed to serialize snapshot: {error}"))?;
    write_text(&path, &text)?;
    Ok(path)
}

/// Load a previously saved snapshot.
pub fn load_snapshot(repo_root: &Path, label: &str) -> Result<BenchSnapshot, String> {
    let path = snapshot_path(repo_root, label)?;
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| {
        format!(
            "failed to parse {}: {error}; save a fresh baseline with the current harness",
            path.display()
        )
    })
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

Do **not** record hostnames, usernames, paths, or other machine-identifying metadata.

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

    let entry = format_history_entry(title, group, runs)?;
    let updated = insert_history_entry(&existing, &entry);
    write_text(&path, &updated)?;
    Ok(path)
}

fn format_history_entry(
    title: &str,
    group: Option<&str>,
    runs: &[BenchRun],
) -> Result<String, String> {
    let date = today_iso_date();
    let group_label = group.unwrap_or("all");
    let mut out = String::new();
    out.push_str(&format!("## {date} — {title}\n\n"));
    out.push_str(&format!("- Group: `{group_label}`\n"));
    out.push_str("- Suite: [`suite.toml`](suite.toml)\n\n");
    out.push_str("| bench | samples | median_ms | min_ms | max_ms | args |\n");
    out.push_str("|-------|---------|-----------|--------|--------|------|\n");
    for run in runs {
        let stats = run.summary()?;
        let args = serde_json::to_string(&run.spec.args)
            .map_err(|error| format!("failed to serialize workload arguments: {error}"))?
            .replace('|', "\\|");
        out.push_str(&format!(
            "| {} | {} | {:.1} | {} | {} | {} |\n",
            run.spec.id, stats.count, stats.median_ms, stats.min_ms, stats.max_ms, args
        ));
    }
    out.push('\n');
    Ok(out)
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

#[cfg(test)]
mod tests {
    use super::{BenchSnapshot, HISTORY_HEADER, format_history_entry, insert_history_entry};
    use crate::suite::fixture_run;
    use std::error::Error;

    #[test]
    fn snapshot_json_preserves_selected_group() -> Result<(), Box<dyn Error>> {
        let runs = [fixture_run("length", &[701, 640, 660, 990, 680])];
        let snapshot = BenchSnapshot::from_runs("baseline", 1, Some("vm"), &runs);
        let json = serde_json::to_string(&snapshot)?;
        let decoded: BenchSnapshot = serde_json::from_str(&json)?;

        assert_eq!(decoded.group.as_deref(), Some("vm"));
        assert_eq!(decoded.benches[0].spec.args, ["100"]);
        assert_eq!(
            decoded.benches[0]
                .samples
                .iter()
                .map(|sample| sample.elapsed_ms)
                .collect::<Vec<_>>(),
            [701, 640, 660, 990, 680]
        );
        assert_eq!(decoded.benches[0].summary()?.median_ms, 680.0);
        Ok(())
    }

    #[test]
    fn insert_history_entry_prepends_after_header_marker() {
        let existing = format!("{HISTORY_HEADER}\n## old\n\n");
        let entry = "## new\n\n";
        let updated = insert_history_entry(&existing, entry);
        assert!(matches!(
            (updated.find("## new"), updated.find("## old")),
            (Some(new_pos), Some(old_pos)) if new_pos < old_pos
        ));
        assert!(updated.contains("Newest entries are prepended below this header."));
    }

    #[test]
    fn format_history_entry_builds_markdown_table() {
        let runs = vec![fixture_run("string_length", &[92, 90, 91, 94, 93])];
        let text = format_history_entry("after char_len cache", Some("vm"), &runs).unwrap();
        assert!(text.contains("## "));
        assert!(text.contains("after char_len cache"));
        assert!(text.contains("| string_length | 5 | 92.0 | 90 | 94 |"));
        assert!(text.contains("Group: `vm`"));
    }
}
