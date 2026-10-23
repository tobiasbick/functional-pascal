//! Deterministic findings derived from profile counters and samples.

use fpas_bytecode::profile::Counter;

use super::{Evidence, percent};

/// Ordered statements naming the dominant measured costs and a concrete next step.
pub(super) fn derive(evidence: &Evidence<'_>, hot_source: Option<&str>) -> Vec<String> {
    let mut findings = Vec::new();
    let collection = evidence.counter(Counter::CollectionNanos);
    let safe_points = evidence.counter(Counter::SafePointNanos);
    // Batch time excludes safe points, and collection runs inside safe points.
    let busy = evidence.report.total.nanos.saturating_add(safe_points);
    if busy > 0 && collection * 10 >= busy {
        findings.push(format!(
            "Collection took {}% of execution time: {} passes traced {} values. Rerun with a larger \
             --gc-poll-interval or --gc-cell-threshold to confirm that pass frequency is the cost.",
            percent(collection, busy),
            evidence.counter(Counter::CollectionPasses),
            evidence.counter(Counter::CollectionVisited),
        ));
    }
    let publication = safe_points.saturating_sub(collection);
    if busy > 0 && publication * 10 >= busy {
        findings.push(format!(
            "Root publication and safe points outside collection took {}% of execution time.",
            percent(publication, busy),
        ));
    }
    let calls = evidence.counter(Counter::ClassifyCalls);
    let visited = evidence.counter(Counter::ClassifyVisited);
    if calls > 0 && visited / calls >= 1000 {
        findings.push(format!(
            "Ownership classification visited {} allocations per call on average over {calls} \
             calls. Storing into a large aggregate that contains functions or cells revalidates \
             its whole graph on each store.",
            visited / calls,
        ));
    }
    let spawns = evidence.counter(Counter::TaskSpawns);
    let yields = evidence.counter(Counter::TaskYields);
    if spawns > 0 && yields > spawns.saturating_mul(100) {
        findings.push(format!(
            "Tasks yielded their time slice {yields} times for {spawns} spawned tasks; each yield \
             saves the task and passes through the shared ready queue ({} contended locks).",
            evidence.counter(Counter::QueueContended),
        ));
    }
    let idle = evidence.counter(Counter::WorkerIdleNanos);
    let workers = std::thread::available_parallelism()
        .map_or(1, std::num::NonZeroUsize::get)
        .saturating_sub(1)
        .max(1) as u64;
    let available = u64::try_from(evidence.wall.as_nanos())
        .unwrap_or(u64::MAX)
        .saturating_mul(workers);
    if spawns > 0 && available > 0 && idle * 2 >= available {
        findings.push(format!(
            "Pool workers were idle for {}% of their available time; the program does not keep \
             {workers} workers busy.",
            percent(idle, available),
        ));
    }
    let total = evidence.report.total;
    if let Some(routine) = evidence.report.routines.first() {
        let share = percent(routine.self_sample.nanos, total.nanos);
        if share >= 50.0 {
            let location = match (hot_source, routine.line) {
                (Some(source), Some(line)) => format!(" ({source}:{line})"),
                _ => String::new(),
            };
            findings.push(format!(
                "Routine `{}`{location} took {share}% of sampled execution time; optimize it first.",
                routine.name,
            ));
        }
    }
    // Compare time per instruction with the run average to expose costly operations.
    for routine in evidence.report.routines.iter().take(5) {
        let sample = routine.self_sample;
        if sample.instructions == 0 || total.instructions == 0 || total.nanos == 0 {
            continue;
        }
        let ratio = (u128::from(sample.nanos) * u128::from(total.instructions))
            / (u128::from(total.nanos) * u128::from(sample.instructions));
        if ratio >= 10 && sample.nanos * 10 >= total.nanos {
            findings.push(format!(
                "Routine `{}` spends {ratio} times the average time per instruction ({}% of time, \
                 {}% of instructions); its operations allocate, validate graphs or call native \
                 code. Inspect its top lines.",
                routine.name,
                percent(sample.nanos, total.nanos),
                percent(sample.instructions, total.instructions),
            ));
        }
    }
    findings
}
