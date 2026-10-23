use super::{Counter, add, begin};

fn count(snapshot: &super::Snapshot, counter: Counter) -> u64 {
    snapshot
        .counts
        .iter()
        .find(|(candidate, _)| *candidate == counter)
        .map_or(0, |(_, value)| *value)
}

// One test owns the process-wide state so parallel test threads cannot interleave.
#[test]
fn runs_reset_sum_threads_ignore_idle_events_and_flag_overlap() {
    let run = begin();
    add(Counter::TaskYields, 2);
    std::thread::scope(|scope| {
        for _ in 0..8 {
            scope.spawn(|| {
                for _ in 0..1000 {
                    add(Counter::TaskYields, 1);
                }
            });
        }
    });
    let snapshot = run.finish();
    assert_eq!(count(&snapshot, Counter::TaskYields), 8002);
    assert!(snapshot.exclusive);
    add(Counter::TaskYields, 5);
    let first = begin();
    assert_eq!(count(&first.finish(), Counter::TaskYields), 0);

    let first = begin();
    add(Counter::TaskSpawns, 1);
    let second = begin();
    add(Counter::TaskSpawns, 1);
    let first = first.finish();
    assert!(!first.exclusive);
    assert_eq!(count(&first, Counter::TaskSpawns), 2);
    assert!(!second.finish().exclusive);
    assert!(begin().finish().exclusive);

    let names: Vec<_> = Counter::ALL.iter().map(|counter| counter.name()).collect();
    assert_eq!(
        names.len(),
        names.iter().collect::<std::collections::HashSet<_>>().len()
    );
}
