//! Count requested live bytes, including reallocations and every runtime thread.
use crate::ALLOCATOR;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Footprint {
    bytes: i128,
    allocations: i128,
}

impl Footprint {
    pub fn now() -> Self {
        let stats = ALLOCATOR.stats();
        // stats_alloc already includes reallocation growth/shrinkage in these byte totals.
        Self {
            bytes: stats.bytes_allocated as i128 - stats.bytes_deallocated as i128,
            allocations: stats.allocations as i128 - stats.deallocations as i128,
        }
    }
}

pub fn isolated(name: &str, run: fn()) {
    if std::env::var("FPAS_GC_MEMORY_CHILD").as_deref() == Ok(name) {
        run();
        println!("GC_MEMORY_CASE_COMPLETED {name}");
        return;
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture", "--test-threads=1"])
        .env("FPAS_GC_MEMORY_CHILD", name)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{name}:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("GC_MEMORY_CASE_COMPLETED {name}\n")),
        "child did not execute the requested memory case: {name}"
    );
    print!("{}", String::from_utf8_lossy(&output.stdout));
}

/// Check each round against one fixed baseline, so small cumulative leaks cannot hide.
pub fn steady_state(name: &str, rounds: usize, mut exercise: impl FnMut()) {
    // Pools can fill over many rounds. Require convergence within a finite warm-up,
    // then compare every measured round to the same baseline without a byte allowance.
    let mut previous = Footprint::now();
    let mut stable = 0;
    let mut warmup = 0;
    while (stable < 4 || warmup < 64) && warmup < 256 {
        exercise();
        let current = Footprint::now();
        stable = if current == previous { stable + 1 } else { 0 };
        previous = current;
        warmup += 1;
    }
    assert!(
        stable >= 4,
        "{name}: memory did not settle within 256 warm-up rounds"
    );
    balanced_rounds(name, warmup, rounds, exercise);
}

fn balanced_rounds(name: &str, warmup: usize, rounds: usize, mut exercise: impl FnMut()) {
    let baseline = Footprint::now();
    let traffic = ALLOCATOR.stats();
    for round in 0..rounds {
        exercise();
        assert_eq!(Footprint::now(), baseline, "{name}, round {round}");
    }
    let traffic = ALLOCATOR.stats() - traffic;
    assert!(traffic.bytes_allocated >= rounds * 32 * 1024);
    println!(
        "{name}: warm-up={warmup}, {rounds} rounds, before bytes={}, after bytes={}, retained bytes=0, retained allocations=0, allocated bytes={}",
        baseline.bytes,
        Footprint::now().bytes,
        traffic.bytes_allocated
    );
}

/// Thread exit destroys thread-local pools; no cache allowance remains afterward.
pub fn completed_threads(name: &str, rounds: usize, exercise: impl Fn() + Sync) {
    // Initialize only Rust's thread machinery, never warm up away a runtime leak.
    std::thread::scope(|scope| scope.spawn(|| {}).join().unwrap());
    balanced_rounds(name, 0, rounds, || {
        std::thread::scope(|scope| scope.spawn(&exercise).join().unwrap());
    });
}

/// Highest sampled growth of live bytes while `run` executes on this thread.
///
/// Sampling can miss short spikes, but a leak grows monotonically and stays visible.
pub fn peak_growth(run: impl FnOnce()) -> i128 {
    let baseline = Footprint::now().bytes;
    let done = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let sampler = scope.spawn(|| {
            let mut peak = baseline;
            while !done.load(Ordering::Acquire) {
                peak = peak.max(Footprint::now().bytes);
                std::thread::sleep(Duration::from_millis(1));
            }
            peak
        });
        run();
        done.store(true, Ordering::Release);
        sampler.join().unwrap() - baseline
    })
}

pub fn verify_counter() {
    std::thread::spawn(|| {}).join().unwrap();
    let baseline = Footprint::now();
    let mut payload = std::thread::spawn(|| vec![0_u8; 256 * 1024])
        .join()
        .unwrap();
    assert_eq!(Footprint::now().bytes - baseline.bytes, 256 * 1024);
    assert_eq!(Footprint::now().allocations - baseline.allocations, 1);
    payload.reserve_exact(256 * 1024);
    assert_eq!(Footprint::now().bytes - baseline.bytes, 512 * 1024);
    payload.shrink_to_fit();
    assert_eq!(Footprint::now().bytes - baseline.bytes, 256 * 1024);
    std::thread::spawn(move || drop(payload)).join().unwrap();
    assert_eq!(Footprint::now(), baseline);
}
