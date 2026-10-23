use super::{Counter, add, disable, enable, snapshot};

fn count(counter: Counter) -> u64 {
    snapshot()
        .into_iter()
        .find(|(candidate, _)| *candidate == counter)
        .map_or(0, |(_, value)| value)
}

// One test owns the process-wide state so parallel test threads cannot interleave.
#[test]
fn counters_reset_sum_threads_and_ignore_disabled_events() {
    enable();
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
    assert_eq!(count(Counter::TaskYields), 8002);
    disable();
    add(Counter::TaskYields, 5);
    assert_eq!(count(Counter::TaskYields), 8002);
    enable();
    assert_eq!(count(Counter::TaskYields), 0);
    disable();
    let names: Vec<_> = Counter::ALL.iter().map(|counter| counter.name()).collect();
    assert_eq!(
        names.len(),
        names.iter().collect::<std::collections::HashSet<_>>().len()
    );
}
