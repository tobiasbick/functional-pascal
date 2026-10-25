//! Deterministic findings derived from profile counters and samples.
//!
//! Each rule reports the measurements that triggered it. Messages state observations;
//! next steps suggest checks and never assert an unmeasured cause.

use fpas_bytecode::profile::Counter;
use fpas_vm::RoutineProfile;

use super::{Evidence, percent};

/// Bound on reported findings; rules run in a fixed order.
const MAX_FINDINGS: usize = 8;

/// One measured value supporting a finding.
pub(super) struct Measure {
    pub name: &'static str,
    pub value: f64,
    pub unit: &'static str,
    /// What a percentage or ratio divides by.
    pub denominator: Option<&'static str>,
}

/// One triggered rule.
pub(super) struct Finding<'a> {
    pub code: &'static str,
    pub message: String,
    pub evidence: Vec<Measure>,
    pub location: Option<&'a RoutineProfile>,
    pub next_steps: Vec<&'static str>,
}

fn measure(name: &'static str, value: f64, unit: &'static str) -> Measure {
    Measure {
        name,
        value,
        unit,
        denominator: None,
    }
}

fn share(name: &'static str, part: u64, whole: u64, denominator: &'static str) -> Measure {
    Measure {
        name,
        value: percent(part, whole),
        unit: "percent",
        denominator: Some(denominator),
    }
}

fn ms(nanos: u64) -> f64 {
    super::round3(nanos as f64 / 1_000_000.0)
}

/// Ordered findings naming the dominant measured costs.
pub(super) fn derive<'a>(evidence: &Evidence<'a>) -> Vec<Finding<'a>> {
    let mut findings = Vec::new();
    let report = evidence.report;
    let total = report.total;
    let collection = evidence.counter(Counter::CollectionNanos);
    let safe_points = evidence.counter(Counter::SafePointNanos);
    // Debugger collection and publication also occur outside sampled dispatch.
    let busy = if evidence.debug {
        u64::try_from(evidence.wall.as_nanos()).unwrap_or(u64::MAX)
    } else {
        total.nanos.saturating_add(safe_points)
    };
    if busy > 0 && collection * 10 >= busy {
        findings.push(Finding {
            code: "collection_time",
            message: format!(
                "Collection took {}% of {}.",
                percent(collection, busy),
                if evidence.debug { "session wall time" } else { "execution time" }
            ),
            evidence: vec![
                share("collection_time_percent", collection, busy, "busy_ms"),
                measure("collection_ms", ms(collection), "ms"),
                measure("busy_ms", ms(busy), "ms"),
                measure(
                    "collection_passes",
                    evidence.counter(Counter::CollectionPasses) as f64,
                    "count",
                ),
                measure(
                    "collection_visited",
                    evidence.counter(Counter::CollectionVisited) as f64,
                    "values",
                ),
            ],
            location: None,
            next_steps: vec![
                "Rerun with a larger --gc-poll-interval or --gc-cell-threshold; if collection time falls, pass frequency is the cost.",
            ],
        });
    }
    let publication = if evidence.debug {
        evidence.counter(Counter::RootPublicationNanos)
    } else {
        safe_points.saturating_sub(collection)
    };
    if busy > 0 && publication * 10 >= busy {
        findings.push(Finding {
            code: if evidence.debug { "debug_root_publication" } else { "safe_point_time" },
            message: format!(
                "{} took {}% of {}.",
                if evidence.debug { "Execution-root publication" } else { "Root publication and safe points outside collection" },
                percent(publication, busy),
                if evidence.debug { "session wall time" } else { "execution time" }
            ),
            evidence: vec![
                share("publication_time_percent", publication, busy, "busy_ms"),
                measure("publication_ms", ms(publication), "ms"),
                measure("busy_ms", ms(busy), "ms"),
            ],
            location: None,
            next_steps: vec![
                "Compare with a smaller live register window or fewer nested calls in the hot routines.",
            ],
        });
    }
    let calls = evidence.counter(Counter::ClassifyCalls);
    let visited = evidence.counter(Counter::ClassifyVisited);
    if calls > 0 && visited / calls >= 1000 {
        findings.push(Finding {
            code: "classification_walks",
            message: format!(
                "Ownership classification visited {} values per call on average.",
                visited / calls
            ),
            evidence: vec![
                measure("classify_calls", calls as f64, "count"),
                measure("classify_visited", visited as f64, "values"),
                measure("visits_per_call", (visited / calls) as f64, "values"),
            ],
            location: None,
            next_steps: vec![
                "Values containing functions or cells are classified when stored or transferred; vary the stored value's graph size and inspect the hot call sites.",
            ],
        });
    }
    let spawns = evidence.counter(Counter::TaskSpawns);
    let yields = evidence.counter(Counter::TaskYields);
    if spawns > 0 && yields > spawns.saturating_mul(100) {
        findings.push(Finding {
            code: "task_yields",
            message: format!(
                "Tasks yielded their time slice {} times per spawned task.",
                yields / spawns
            ),
            evidence: vec![
                measure("task_yields", yields as f64, "count"),
                measure("task_spawns", spawns as f64, "count"),
                measure(
                    "queue_contended",
                    evidence.counter(Counter::QueueContended) as f64,
                    "count",
                ),
            ],
            location: None,
            next_steps: vec!["Compare wall time with fewer concurrently queued tasks."],
        });
    }
    let idle = evidence.counter(Counter::WorkerIdleNanos);
    let workers = evidence.workers as u64;
    let available = u64::try_from(evidence.wall.as_nanos())
        .unwrap_or(u64::MAX)
        .saturating_mul(workers);
    if spawns > 0 && available > 0 && idle * 2 >= available {
        findings.push(Finding {
            code: "worker_idle",
            message: format!(
                "Pool workers were idle for {}% of their available time.",
                percent(idle, available)
            ),
            evidence: vec![
                share("idle_percent", idle, available, "workers * wall_ms"),
                measure("worker_idle_ms", ms(idle), "ms"),
                measure("workers", workers as f64, "count"),
                measure("parallelism", evidence.parallelism(), "threads"),
            ],
            location: None,
            next_steps: vec![
                "Check whether tasks have unequal amounts of work or a serial phase waits for them; compare parallelism with the worker count.",
            ],
        });
    }
    if let Some(routine) = report.routines.iter().find(|routine| !routine.entry) {
        let time = routine.self_sample.nanos;
        if total.nanos > 0 && time * 2 >= total.nanos {
            findings.push(Finding {
                code: "dominant_routine",
                message: format!(
                    "Routine `{}` took {}% of sampled execution time.",
                    routine.name,
                    percent(time, total.nanos)
                ),
                evidence: vec![
                    share("self_time_percent", time, total.nanos, "sampled_ms"),
                    measure("self_ms", ms(time), "ms"),
                ],
                location: Some(routine),
                next_steps: vec!["Inspect the routine's rows in `lines` first."],
            });
        }
    }
    // Compare time per instruction with the run average to expose costly operations.
    for routine in report.routines.iter().take(5) {
        let sample = routine.self_sample;
        if sample.instructions == 0 || total.instructions == 0 || total.nanos == 0 {
            continue;
        }
        let ratio = (u128::from(sample.nanos) * u128::from(total.instructions))
            / (u128::from(total.nanos) * u128::from(sample.instructions));
        if ratio >= 10 && sample.nanos * 10 >= total.nanos {
            findings.push(Finding {
                code: "costly_instructions",
                message: format!(
                    "Routine `{}` spends {ratio} times the average time per instruction.",
                    routine.name
                ),
                evidence: vec![
                    Measure {
                        name: "time_per_instruction_ratio",
                        value: ratio as f64,
                        unit: "ratio",
                        denominator: Some("run average time per instruction"),
                    },
                    share("self_time_percent", sample.nanos, total.nanos, "sampled_ms"),
                    share(
                        "self_instruction_percent",
                        sample.instructions,
                        total.instructions,
                        "instructions",
                    ),
                ],
                location: Some(routine),
                next_steps: vec![
                    "Check whether its top lines allocate, store into aggregates or call native code.",
                ],
            });
        }
    }
    findings.truncate(MAX_FINDINGS);
    findings
}
