//! `fpas run --profile`: one JSON report of hot routines, lines and runtime costs.
//!
//! Contract: `docs/pascal/tools/profiler.md`.

mod findings;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use fpas_bytecode::profile::{self, Counter};
use fpas_vm::{ProfileReport, ProfileSample, Vm};
use serde_json::{Map, Value as Json, json};

const TOP_ROWS: usize = 25;

const ATTRIBUTION: &str = "Each dispatch batch (at most 256 instructions) is attributed to the \
line and routine active when it ends; inclusive totals add every distinct routine on the call \
stack. Milliseconds are summed over all worker threads and exclude waits outside FPAS code.";

/// Profiling state of one `fpas run`.
pub(crate) struct Session {
    destination: PathBuf,
    started: Instant,
}

impl Session {
    /// Enable process-wide counters and VM sampling before execution starts.
    pub(crate) fn start(vm: &mut Vm, destination: PathBuf) -> Self {
        profile::enable();
        vm.enable_profile();
        Self {
            destination,
            started: Instant::now(),
        }
    }

    /// Stop counting and publish the report to the validated new destination.
    pub(crate) fn finish(
        self,
        vm: &Vm,
        exit_code: i32,
        sources: Option<&[PathBuf]>,
    ) -> std::io::Result<()> {
        let wall = self.started.elapsed();
        profile::disable();
        let counters = profile::snapshot();
        let report = vm.profile_report().unwrap_or_default();
        let document = render(&report, &counters, wall, exit_code, sources);
        let mut bytes = serde_json::to_vec_pretty(&document)?;
        bytes.push(b'\n');
        crate::cli_failure_dump::publish_report(&self.destination, &bytes)
    }

    /// Destination shown in publication errors.
    pub(crate) fn destination(&self) -> &Path {
        &self.destination
    }
}

/// Totals and counters the finding rules read.
pub(crate) struct Evidence<'a> {
    pub report: &'a ProfileReport,
    pub counters: &'a [(Counter, u64)],
    pub wall: Duration,
}

impl Evidence<'_> {
    pub(crate) fn counter(&self, counter: Counter) -> u64 {
        self.counters
            .iter()
            .find(|(candidate, _)| *candidate == counter)
            .map_or(0, |(_, value)| *value)
    }
}

fn render(
    report: &ProfileReport,
    counters: &[(Counter, u64)],
    wall: Duration,
    exit_code: i32,
    sources: Option<&[PathBuf]>,
) -> Json {
    let total = report.total;
    let source = |index: Option<u32>| -> Json {
        index
            .and_then(|index| sources?.get(usize::try_from(index).ok()?))
            .map_or(Json::Null, |path| json!(display_path(path)))
    };
    let routines: Vec<Json> = report
        .routines
        .iter()
        .filter(|routine| routine.inclusive.instructions > 0)
        .take(TOP_ROWS)
        .map(|routine| {
            json!({
                "name": routine.name,
                "source": source(routine.source),
                "line": routine.line,
                "self_instructions": routine.self_sample.instructions,
                "self_percent": percent(routine.self_sample.instructions, total.instructions),
                "self_ms": millis(routine.self_sample),
                "inclusive_instructions": routine.inclusive.instructions,
                "inclusive_percent": percent(routine.inclusive.instructions, total.instructions),
                "inclusive_ms": millis(routine.inclusive),
            })
        })
        .collect();
    let lines: Vec<Json> = report
        .lines
        .iter()
        .filter(|line| line.sample.instructions > 0)
        .take(TOP_ROWS)
        .map(|line| {
            json!({
                "source": source(line.source),
                "line": line.line,
                "routine": line.routine,
                "instructions": line.sample.instructions,
                "percent": percent(line.sample.instructions, total.instructions),
                "ms": millis(line.sample),
            })
        })
        .collect();
    let counter_map: Map<String, Json> = counters
        .iter()
        .map(|(counter, value)| (counter.name().to_owned(), json!(value)))
        .collect();
    let evidence = Evidence {
        report,
        counters,
        wall,
    };
    let hot_source = report
        .routines
        .first()
        .and_then(|routine| source(routine.source).as_str().map(str::to_owned));
    json!({
        "format": "fpas-profile",
        "version": 1,
        "status": if exit_code == 0 { "ok" } else { "error" },
        "exit_code": exit_code,
        "wall_ms": round3(wall.as_secs_f64() * 1000.0),
        "instructions": total.instructions,
        "sampled_ms": millis(total),
        "attribution": ATTRIBUTION,
        "routines": routines,
        "lines": lines,
        "counters": counter_map,
        "findings": findings::derive(&evidence, hot_source.as_deref()),
    })
}

fn display_path(path: &Path) -> String {
    std::env::current_dir()
        .ok()
        .and_then(|cwd| path.strip_prefix(cwd).ok().map(Path::to_path_buf))
        .unwrap_or_else(|| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
}

pub(crate) fn percent(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        return 0.0;
    }
    (part as f64 * 1000.0 / whole as f64).round() / 10.0
}

fn millis(sample: ProfileSample) -> f64 {
    round3(sample.nanos as f64 / 1_000_000.0)
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests;
