//! Validate benchmark baselines and build complete comparison rows.

use super::BenchSnapshot;
use crate::suite::BenchRun;
use std::collections::HashMap;

/// A validated baseline indexed by benchmark id.
#[derive(Debug)]
pub struct ComparisonBaseline {
    label: String,
    elapsed_ms_by_id: HashMap<String, u64>,
}

impl ComparisonBaseline {
    /// Validate snapshot group identity and reject duplicate benchmark ids.
    pub fn new(snapshot: &BenchSnapshot, requested_group: Option<&str>) -> Result<Self, String> {
        if snapshot.group.as_deref() != requested_group {
            return Err(group_mismatch_error(snapshot, requested_group));
        }

        let mut elapsed_ms_by_id = HashMap::with_capacity(snapshot.benches.len());
        for result in &snapshot.benches {
            if elapsed_ms_by_id
                .insert(result.id.clone(), result.elapsed_ms)
                .is_some()
            {
                return Err(format!(
                    "baseline snapshot `{}` contains duplicate benchmark id `{}`; save the baseline again before comparing",
                    snapshot.label, result.id
                ));
            }
        }

        Ok(Self {
            label: snapshot.label.clone(),
            elapsed_ms_by_id,
        })
    }

    /// Compare current runs after requiring one baseline result for every benchmark id.
    pub fn compare(&self, current: &[BenchRun]) -> Result<Vec<CompareRow>, String> {
        let missing: Vec<&str> = current
            .iter()
            .filter(|run| !self.elapsed_ms_by_id.contains_key(&run.id))
            .map(|run| run.id.as_str())
            .collect();
        if !missing.is_empty() {
            return Err(format!(
                "baseline snapshot `{}` is missing current benchmark id(s): {}; save the baseline again with the same group selection",
                self.label,
                quote_ids(&missing)
            ));
        }

        current.iter().map(|run| self.compare_run(run)).collect()
    }

    fn compare_run(&self, run: &BenchRun) -> Result<CompareRow, String> {
        let baseline_ms = self.elapsed_ms_by_id.get(&run.id).copied().ok_or_else(|| {
            format!(
                "baseline snapshot `{}` is missing current benchmark id `{}`",
                self.label, run.id
            )
        })?;
        let delta_pct = match (baseline_ms, run.elapsed_ms) {
            (0, 0) => 0.0,
            (0, current_ms) => {
                return Err(format!(
                    "baseline snapshot `{}` reports 0 ms for benchmark `{}`, but the current run reports {current_ms} ms; increase the benchmark workload and save a new baseline",
                    self.label, run.id
                ));
            }
            (baseline_ms, current_ms) => {
                ((current_ms as f64) - (baseline_ms as f64)) / (baseline_ms as f64) * 100.0
            }
        };

        Ok(CompareRow {
            id: run.id.clone(),
            baseline_ms,
            current_ms: run.elapsed_ms,
            delta_pct,
            throughput: run.throughput.clone(),
        })
    }
}

/// One complete comparison row for console output and regression gating.
#[derive(Debug, Clone)]
pub struct CompareRow {
    /// Benchmark id.
    pub id: String,
    /// Baseline elapsed milliseconds.
    pub baseline_ms: u64,
    /// Current elapsed milliseconds.
    pub current_ms: u64,
    /// Percent change versus baseline: `(current - baseline) / baseline * 100`.
    pub delta_pct: f64,
    /// Optional throughput from the current run.
    pub throughput: Option<String>,
}

/// Return whether any compared benchmark exceeds `threshold_pct` slowdown.
pub fn has_regression(rows: &[CompareRow], threshold_pct: f64) -> bool {
    rows.iter().any(|row| row.delta_pct > threshold_pct)
}

fn group_mismatch_error(snapshot: &BenchSnapshot, requested_group: Option<&str>) -> String {
    let snapshot_group = snapshot.group.as_deref().unwrap_or("all");
    let requested_group_label = requested_group.unwrap_or("all");
    let save_command = match requested_group {
        Some(group) => format!("cargo bench-fpas save {} --group {group}", snapshot.label),
        None => format!("cargo bench-fpas save {}", snapshot.label),
    };
    format!(
        "baseline snapshot `{}` was saved for group `{snapshot_group}`, but compare requested `{requested_group_label}`; replace it with `{save_command}`",
        snapshot.label
    )
}

fn quote_ids(ids: &[&str]) -> String {
    ids.iter()
        .map(|id| format!("`{id}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::ComparisonBaseline;
    use crate::results::{BenchResult, BenchSnapshot};
    use crate::suite::BenchRun;

    fn snapshot(group: Option<&str>, benches: &[(&str, u64)]) -> BenchSnapshot {
        BenchSnapshot {
            label: "baseline".to_owned(),
            timestamp_unix: 0,
            group: group.map(str::to_owned),
            benches: benches
                .iter()
                .map(|(id, elapsed_ms)| BenchResult {
                    id: (*id).to_owned(),
                    elapsed_ms: *elapsed_ms,
                    throughput: None,
                    raw_stdout: String::new(),
                })
                .collect(),
        }
    }

    fn runs(benches: &[(&str, u64)]) -> Vec<BenchRun> {
        benches
            .iter()
            .map(|(id, elapsed_ms)| BenchRun {
                id: (*id).to_owned(),
                elapsed_ms: *elapsed_ms,
                throughput: None,
                raw_stdout: String::new(),
            })
            .collect()
    }

    #[test]
    fn comparison_rejects_disjoint_baseline() {
        let baseline = snapshot(None, &[("old", 10)]);
        let result = ComparisonBaseline::new(&baseline, None)
            .and_then(|baseline| baseline.compare(&runs(&[("first", 11), ("second", 12)])));

        assert_eq!(
            result.map(|rows| rows.len()),
            Err("baseline snapshot `baseline` is missing current benchmark id(s): `first`, `second`; save the baseline again with the same group selection".to_owned())
        );
    }

    #[test]
    fn comparison_rejects_partially_overlapping_baseline() {
        let baseline = snapshot(None, &[("first", 10)]);
        let result = ComparisonBaseline::new(&baseline, None)
            .and_then(|baseline| baseline.compare(&runs(&[("first", 11), ("second", 12)])));

        assert_eq!(
            result.map(|rows| rows.len()),
            Err("baseline snapshot `baseline` is missing current benchmark id(s): `second`; save the baseline again with the same group selection".to_owned())
        );
    }

    #[test]
    fn comparison_rejects_duplicate_baseline_id() {
        let baseline = snapshot(None, &[("integer_loop", 10), ("integer_loop", 12)]);

        assert_eq!(
            ComparisonBaseline::new(&baseline, None).map(|_| ()),
            Err("baseline snapshot `baseline` contains duplicate benchmark id `integer_loop`; save the baseline again before comparing".to_owned())
        );
    }

    #[test]
    fn comparison_rejects_mismatched_group() {
        let baseline = snapshot(Some("vm"), &[("integer_loop", 10)]);

        assert_eq!(
            ComparisonBaseline::new(&baseline, Some("tui")).map(|_| ()),
            Err("baseline snapshot `baseline` was saved for group `vm`, but compare requested `tui`; replace it with `cargo bench-fpas save baseline --group tui`".to_owned())
        );
    }

    #[test]
    fn comparison_builds_complete_rows_for_matching_group() {
        let baseline = snapshot(Some("vm"), &[("integer_loop", 10)]);
        let result = ComparisonBaseline::new(&baseline, Some("vm"))
            .and_then(|baseline| baseline.compare(&runs(&[("integer_loop", 12)])))
            .map(|rows| {
                rows.into_iter()
                    .next()
                    .map(|row| (row.baseline_ms, row.current_ms, row.delta_pct))
            });

        assert_eq!(result, Ok(Some((10, 12, 20.0))));
    }

    #[test]
    fn comparison_defines_zero_to_zero_as_no_change() {
        let baseline = snapshot(None, &[("fast", 0)]);
        let result = ComparisonBaseline::new(&baseline, None)
            .and_then(|baseline| baseline.compare(&runs(&[("fast", 0)])))
            .map(|rows| rows.into_iter().next().map(|row| row.delta_pct));

        assert_eq!(result, Ok(Some(0.0)));
    }

    #[test]
    fn comparison_rejects_zero_to_positive_duration() {
        let baseline = snapshot(None, &[("fast", 0)]);
        let result = ComparisonBaseline::new(&baseline, None)
            .and_then(|baseline| baseline.compare(&runs(&[("fast", 1)])));

        assert_eq!(
            result.map(|rows| rows.len()),
            Err("baseline snapshot `baseline` reports 0 ms for benchmark `fast`, but the current run reports 1 ms; increase the benchmark workload and save a new baseline".to_owned())
        );
    }
}
