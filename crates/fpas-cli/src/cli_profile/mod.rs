//! `fpas run --profile`: one JSON report of hot routines, lines and runtime costs.
//!
//! Contract: `docs/pascal/tools/profiler.md`.

mod findings;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use fpas_bytecode::CollectionSchedule;
use fpas_bytecode::profile::{self, Counter};
use fpas_vm::{ProfileReport, ProfileSample, Vm};
use serde_json::{Map, Value as Json, json};

const TOP_ROWS: usize = 25;
const FORMAT_VERSION: u32 = 2;

/// What was executed, without absolute paths or program arguments.
pub(crate) struct Target {
    /// `source`, `project`, `workspace` or `compiled`.
    pub kind: &'static str,
    /// Path relative to the current directory, else the file name.
    pub name: String,
    /// BLAKE3 digest of the executed bytecode; `None` when it cannot be encoded.
    pub digest: Option<String>,
}

impl Target {
    pub(crate) fn new(
        kind: &'static str,
        path: &Path,
        executable: &fpas_bytecode::Executable,
    ) -> Self {
        Self {
            kind,
            name: display_path(path),
            digest: fpas_program::executable_digest(executable)
                .ok()
                .map(|digest| format!("blake3:{digest:?}")),
        }
    }
}

/// Profiling state of one `fpas run`.
pub(crate) struct Session {
    destination: PathBuf,
    target: Target,
    collection: CollectionSchedule,
    counting: profile::Run,
    started: Instant,
}

impl Session {
    /// Start counters and VM sampling before execution starts.
    pub(crate) fn start(
        vm: &mut Vm,
        destination: PathBuf,
        target: Target,
        collection: CollectionSchedule,
    ) -> Self {
        let counting = profile::begin();
        vm.enable_profile();
        Self {
            destination,
            target,
            collection,
            counting,
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
        let counters = self.counting.finish();
        let report = vm.profile_report().unwrap_or_default();
        let run = Run {
            target: &self.target,
            collection: self.collection,
            workers: vm.worker_count(),
            counters_exclusive: counters.exclusive,
        };
        let document = render(&report, &counters.counts, wall, &run, exit_code, sources);
        let mut bytes = serde_json::to_vec_pretty(&document)?;
        bytes.push(b'\n');
        crate::cli_failure_dump::publish_report(&self.destination, &bytes)
    }

    /// Destination shown in publication errors.
    pub(crate) fn destination(&self) -> &Path {
        &self.destination
    }
}

/// Execution settings recorded for comparing reports.
struct Run<'a> {
    target: &'a Target,
    collection: CollectionSchedule,
    workers: usize,
    counters_exclusive: bool,
}

/// Totals and counters the finding rules read.
pub(crate) struct Evidence<'a> {
    pub report: &'a ProfileReport,
    pub counters: &'a [(Counter, u64)],
    pub wall: Duration,
    /// Pool threads of the profiled VM.
    pub workers: usize,
}

impl Evidence<'_> {
    pub(crate) fn counter(&self, counter: Counter) -> u64 {
        self.counters
            .iter()
            .find(|(candidate, _)| *candidate == counter)
            .map_or(0, |(_, value)| *value)
    }

    /// Average number of threads executing FPAS code during the run.
    pub(crate) fn parallelism(&self) -> f64 {
        let wall = self.wall.as_nanos();
        if wall == 0 {
            return 0.0;
        }
        (self.report.total.nanos as f64 * 100.0 / wall as f64).round() / 100.0
    }
}

fn render(
    report: &ProfileReport,
    counters: &[(Counter, u64)],
    wall: Duration,
    run: &Run<'_>,
    exit_code: i32,
    sources: Option<&[PathBuf]>,
) -> Json {
    let total = report.total;
    let source = |index: Option<u32>| -> Json {
        index
            .and_then(|index| sources?.get(usize::try_from(index).ok()?))
            .map_or(Json::Null, |path| json!(display_path(path)))
    };
    let routine_rows: Vec<_> = report
        .routines
        .iter()
        .filter(|routine| routine.inclusive.instructions > 0)
        .collect();
    let routines: Vec<Json> = routine_rows
        .iter()
        .take(TOP_ROWS)
        .map(|routine| {
            json!({
                "name": routine.name,
                "source": source(routine.source),
                "line": routine.line,
                "self_instructions": routine.self_sample.instructions,
                "inclusive_instructions": routine.inclusive.instructions,
                "self_ms": millis(routine.self_sample),
                "inclusive_ms": millis(routine.inclusive),
                "self_time_percent": percent(routine.self_sample.nanos, total.nanos),
                "inclusive_time_percent": percent(routine.inclusive.nanos, total.nanos),
            })
        })
        .collect();
    let line_rows: Vec<_> = report
        .lines
        .iter()
        .filter(|line| line.sample.instructions > 0)
        .collect();
    let lines: Vec<Json> = line_rows
        .iter()
        .take(TOP_ROWS)
        .map(|line| {
            json!({
                "source": source(line.source),
                "line": line.line,
                "routine": line.routine,
                "instructions": line.sample.instructions,
                "ms": millis(line.sample),
                "time_percent": percent(line.sample.nanos, total.nanos),
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
        workers: run.workers,
    };
    let findings: Vec<Json> = findings::derive(&evidence)
        .into_iter()
        .map(|finding| {
            json!({
                "code": finding.code,
                "message": finding.message,
                "evidence": finding.evidence.iter().map(|measure| json!({
                    "name": measure.name,
                    "value": measure.value,
                    "unit": measure.unit,
                    "denominator": measure.denominator,
                })).collect::<Vec<_>>(),
                "location": finding.location.map_or(Json::Null, |routine| json!({
                    "routine": routine.name,
                    "source": source(routine.source),
                    "line": routine.line,
                })),
                "next_steps": finding.next_steps,
            })
        })
        .collect();
    let build = fpas_build::BuildOptions::default().compiler_version;
    json!({
        "format": "fpas-profile",
        "version": FORMAT_VERSION,
        "status": if exit_code == 0 { "ok" } else { "error" },
        "exit_code": exit_code,
        "wall_ms": round3(wall.as_secs_f64() * 1000.0),
        "sampled_ms": millis(total),
        "parallelism": evidence.parallelism(),
        "instructions": total.instructions,
        "run": {
            "tool": {
                "version": env!("CARGO_PKG_VERSION"),
                "build": build.split_once('-').map(|(_, id)| id),
            },
            "target": {
                "kind": run.target.kind,
                "name": run.target.name,
                "digest": run.target.digest,
            },
            "runtime": {
                "workers": run.workers,
                "collection": {
                    "cell_threshold": run.collection.allocations.get(),
                    "mutation_threshold": run.collection.mutations.get(),
                    "poll_interval": run.collection.safe_points.get(),
                },
            },
            "sampling": {
                "method": "dispatch_batch_end",
                "max_batch_instructions": fpas_vm::MAX_BATCH_STEPS,
                "row_limit": TOP_ROWS,
                "counters_exclusive": run.counters_exclusive,
            },
        },
        "routines": { "total": routine_rows.len(), "rows": routines },
        "lines": { "total": line_rows.len(), "rows": lines },
        "counters": counter_map,
        "findings": findings,
    })
}

/// Path relative to the current directory, else only the file name: reports never
/// contain absolute paths.
fn display_path(path: &Path) -> String {
    let shown = std::env::current_dir()
        .ok()
        .and_then(|cwd| path.strip_prefix(cwd).ok().map(Path::to_path_buf))
        .or_else(|| path.file_name().map(PathBuf::from))
        .unwrap_or_default();
    shown.to_string_lossy().replace('\\', "/")
}

/// Share of `whole` in percent with one decimal; zero for an empty whole.
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
