use std::time::Duration;

use fpas_bytecode::profile::Counter;
use fpas_vm::{ProfileReport, ProfileSample, RoutineProfile};

use super::{Evidence, findings, percent};

#[test]
fn percent_rounds_to_one_decimal_and_handles_empty_totals() {
    assert_eq!(percent(1, 3), 33.3);
    assert_eq!(percent(2, 3), 66.7);
    assert_eq!(percent(5, 0), 0.0);
}

fn sample(instructions: u64, millis: u64) -> ProfileSample {
    ProfileSample {
        instructions,
        nanos: millis * 1_000_000,
    }
}

fn routine(name: &str, entry: bool, own: ProfileSample) -> RoutineProfile {
    RoutineProfile {
        name: name.to_owned(),
        entry,
        source: Some(0),
        line: Some(3),
        self_sample: own,
        inclusive: own,
    }
}

fn codes(report: &ProfileReport, counters: &[(Counter, u64)], workers: usize) -> Vec<String> {
    let evidence = Evidence {
        debug: false,
        report,
        counters,
        wall: Duration::from_millis(10),
        workers,
    };
    findings::derive(&evidence)
        .iter()
        .map(|finding| finding.code.to_owned())
        .collect()
}

fn plain(total: ProfileSample) -> ProfileReport {
    ProfileReport {
        total,
        ..ProfileReport::default()
    }
}

#[test]
fn classification_finding_reports_measurements_without_claiming_a_cause() {
    let report = plain(sample(1000, 1));
    let counters = [
        (Counter::ClassifyCalls, 10),
        (Counter::ClassifyVisited, 50_000),
    ];
    let evidence = Evidence {
        debug: false,
        report: &report,
        counters: &counters,
        wall: Duration::from_millis(10),
        workers: 0,
    };
    let findings = findings::derive(&evidence);
    assert_eq!(findings.len(), 1);
    let finding = &findings[0];
    assert_eq!(finding.code, "classification_walks");
    assert!(finding.message.contains("5000 values per call"));
    let per_call = finding
        .evidence
        .iter()
        .find(|measure| measure.name == "visits_per_call")
        .expect("visits per call");
    assert_eq!((per_call.value, per_call.unit), (5000.0, "values"));
    let text = format!("{} {:?}", finding.message, finding.next_steps);
    assert!(!text.contains("whole"), "{text}");
}

#[test]
fn debug_runtime_findings_use_session_wall_time_and_all_root_publications() {
    let report = plain(sample(1000, 1));
    let counters = [
        (Counter::CollectionNanos, 30_000_000),
        (Counter::RootPublicationNanos, 50_000_000),
    ];
    let evidence = Evidence {
        debug: true,
        report: &report,
        counters: &counters,
        wall: Duration::from_millis(100),
        workers: 0,
    };
    let findings = findings::derive(&evidence);
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].code, "collection_time");
    assert_eq!(findings[0].evidence[0].value, 30.0);
    assert_eq!(findings[1].code, "debug_root_publication");
    assert_eq!(findings[1].evidence[0].value, 50.0);
    assert_eq!(findings[1].evidence[2].value, 100.0);
}

#[test]
fn idle_finding_uses_the_vm_pool_size() {
    let report = plain(sample(1000, 1));
    let idle = [
        (Counter::TaskSpawns, 4),
        (Counter::WorkerIdleNanos, 30_000_000),
    ];
    // 30 ms idle over a 10 ms run: 75 % of four workers, impossible for two.
    assert_eq!(codes(&report, &idle, 4), ["worker_idle"]);
    assert!(codes(&report, &idle, 0).is_empty());
}

#[test]
fn dominant_routine_skips_the_entry_routine_and_reports_its_location() {
    let mut report = plain(sample(1000, 10));
    report.routines = vec![
        routine("$entry_main", true, sample(900, 9)),
        routine("work", false, sample(100, 1)),
    ];
    assert!(codes(&report, &[], 0).is_empty());
    report.routines[1].self_sample = sample(600, 6);
    report.routines.swap(0, 1);
    let evidence = Evidence {
        debug: false,
        report: &report,
        counters: &[],
        wall: Duration::from_millis(10),
        workers: 0,
    };
    let findings = findings::derive(&evidence);
    assert_eq!(findings[0].code, "dominant_routine");
    assert_eq!(
        findings[0].location.map(|routine| routine.name.as_str()),
        Some("work")
    );
}

#[test]
fn rules_stay_silent_on_empty_measurements() {
    let report = ProfileReport::default();
    let evidence = Evidence {
        debug: false,
        report: &report,
        counters: &[(Counter::TaskSpawns, 1)],
        wall: Duration::ZERO,
        workers: 4,
    };
    assert!(findings::derive(&evidence).is_empty());
    assert_eq!(evidence.parallelism(), 0.0);
}
