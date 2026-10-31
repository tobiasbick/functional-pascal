//! Repeated workload measurements and descriptive timing statistics.

use super::BenchSpec;
use serde::{Deserialize, Serialize};

/// One complete workload, including its configuration and every captured sample.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchRun {
    pub spec: BenchSpec,
    /// Samples stay in execution order, including outliers.
    pub samples: Vec<BenchSample>,
}

/// Timed output from one fresh benchmark process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchSample {
    pub elapsed_ms: u64,
    pub throughput: Option<String>,
    pub raw_stdout: String,
}

/// Median and observed range; the range is not a confidence interval.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimingSummary {
    pub count: usize,
    pub median_ms: f64,
    pub min_ms: u64,
    pub max_ms: u64,
}

impl BenchRun {
    pub fn summary(&self) -> Result<TimingSummary, String> {
        if self.samples.len() != self.spec.samples.get() {
            return Err(format!(
                "benchmark `{}` requires {} samples, found {}; save a fresh baseline",
                self.spec.id,
                self.spec.samples,
                self.samples.len()
            ));
        }
        let mut times: Vec<_> = self
            .samples
            .iter()
            .map(|sample| sample.elapsed_ms)
            .collect();
        times.sort_unstable();
        let count = times.len();
        let middle = count / 2;
        let median_ms = if count.is_multiple_of(2) {
            times[middle - 1] as f64 / 2.0 + times[middle] as f64 / 2.0
        } else {
            times[middle] as f64
        };
        Ok(TimingSummary {
            count,
            median_ms,
            min_ms: times[0],
            max_ms: times[count - 1],
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::suite::BenchDriver;

    pub(crate) fn fixture_run(id: &str, times: &[u64]) -> BenchRun {
        BenchRun {
            spec: BenchSpec {
                id: id.to_owned(),
                group: "vm".to_owned(),
                driver: BenchDriver::Fpas,
                path: "fixture.fpas".to_owned(),
                args: vec!["100".to_owned()],
                timeout_ms: 1000,
                samples: times.len().try_into().unwrap(),
            },
            samples: times
                .iter()
                .map(|&elapsed_ms| BenchSample {
                    elapsed_ms,
                    throughput: None,
                    raw_stdout: format!("elapsed: {elapsed_ms} ms\n"),
                })
                .collect(),
        }
    }

    #[test]
    fn summarizes_observed_median_and_range_without_discarding_outliers() {
        for (times, expected) in [
            (
                vec![900, 600, 700, 650, 950],
                TimingSummary {
                    count: 5,
                    median_ms: 700.0,
                    min_ms: 600,
                    max_ms: 950,
                },
            ),
            (
                vec![800, 601],
                TimingSummary {
                    count: 2,
                    median_ms: 700.5,
                    min_ms: 601,
                    max_ms: 800,
                },
            ),
            (
                vec![0],
                TimingSummary {
                    count: 1,
                    median_ms: 0.0,
                    min_ms: 0,
                    max_ms: 0,
                },
            ),
        ] {
            assert_eq!(fixture_run("sample", &times).summary(), Ok(expected));
        }
    }

    #[test]
    fn rejects_incomplete_measurements() {
        let mut run = fixture_run("sample", &[600, 700, 800, 900, 1000]);
        run.samples.pop();
        assert!(
            run.summary()
                .unwrap_err()
                .contains("requires 5 samples, found 4")
        );
    }
}
