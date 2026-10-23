use std::time::Duration;

use fpas_bytecode::profile::Counter;
use fpas_vm::{ProfileReport, ProfileSample};

use super::{Evidence, findings, percent};

#[test]
fn percent_rounds_to_one_decimal_and_handles_empty_totals() {
    assert_eq!(percent(1, 3), 33.3);
    assert_eq!(percent(2, 3), 66.7);
    assert_eq!(percent(5, 0), 0.0);
}

fn derive(counters: &[(Counter, u64)], workers: usize) -> Vec<String> {
    let report = ProfileReport {
        total: ProfileSample {
            instructions: 1000,
            nanos: 1_000_000,
        },
        ..ProfileReport::default()
    };
    findings::derive(
        &Evidence {
            report: &report,
            counters,
            wall: Duration::from_millis(10),
            workers,
        },
        None,
    )
}

#[test]
fn classification_finding_reports_measurements_without_claiming_a_cause() {
    let findings = derive(
        &[
            (Counter::ClassifyCalls, 10),
            (Counter::ClassifyVisited, 50_000),
        ],
        0,
    );
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(findings[0].contains("5000 allocations per call"));
    assert!(!findings[0].contains("whole graph"));
}

#[test]
fn idle_finding_uses_the_vm_pool_size() {
    let idle = [
        (Counter::TaskSpawns, 4),
        (Counter::WorkerIdleNanos, 30_000_000),
    ];
    // 30 ms idle over 10 ms: busy with 2 workers is impossible, idle with 4 is 75 %.
    assert!(derive(&idle, 4).iter().any(|text| text.contains("75%")));
    assert!(derive(&idle, 0).iter().all(|text| !text.contains("idle")));
}
