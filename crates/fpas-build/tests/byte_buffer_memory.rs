//! Actual allocation balance for byte payloads across cycles, tasks and VM failure.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "isolated allocation regressions use direct assertions"
)]

use fpas_build::{BuildOptions, check_program};
use fpas_runtime::{CollectionSchedule, execution::Vm};
use stats_alloc::{INSTRUMENTED_SYSTEM, StatsAlloc};
use std::{
    alloc::System,
    num::NonZeroUsize,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[global_allocator]
static ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn footprint() -> (i128, i128) {
    let stats = ALLOCATOR.stats();
    (
        stats.bytes_allocated as i128 - stats.bytes_deallocated as i128,
        stats.allocations as i128 - stats.deallocations as i128,
    )
}

#[test]
fn native_bytes_cycles_tasks_and_abandonment_release_all_allocations() {
    const CASE: &str = "native_bytes_cycles_tasks_and_abandonment_release_all_allocations";
    if std::env::var("FPAS_BYTE_MEMORY_CHILD").as_deref() != Ok(CASE) {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CASE, "--nocapture", "--test-threads=1"])
            .env("FPAS_BYTE_MEMORY_CHILD", CASE)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(90);
        while child.try_wait().unwrap().is_none() {
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("byte-buffer allocation fixture exceeded its cleanup deadline");
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("BYTE_MEMORY_COMPLETED"));
        return;
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let entry = root.join("tests/stdlib/bytes/lifetime_test.fpas");
    let source = std::fs::read_to_string(&entry).unwrap();
    let library = fpas_project::load_standard_library(&root.join("lib")).unwrap();
    let graph = fpas_project::prepare_program_unit_graph(&[], &Default::default(), Some(&library))
        .unwrap()
        .instantiate(&entry);
    for failing in [false, true] {
        let source = if failing {
            source.replace("Exercise(false);", "Exercise(true);")
        } else {
            source.clone()
        };
        let (program, errors) = fpas_parser::parse(&source);
        assert!(errors.is_empty(), "{errors:?}");
        let selection =
            fpas_project::resolve_program_units(&graph, &program.uses, Some(&entry)).unwrap();
        let image = check_program(&graph, &selection, &program, &BuildOptions::default())
            .unwrap()
            .executable;
        for schedule in [
            CollectionSchedule::default(),
            CollectionSchedule {
                allocations: NonZeroUsize::new(1).unwrap(),
                mutations: NonZeroUsize::new(1).unwrap(),
                safe_points: NonZeroUsize::new(1).unwrap(),
            },
        ] {
            std::thread::scope(|scope| scope.spawn(|| {}).join().unwrap());
            let baseline = footprint();
            for round in 0..4 {
                std::thread::scope(|scope| {
                    scope
                        .spawn(|| {
                            let outcome = Vm::new(image.clone())
                                .with_collection_schedule(schedule)
                                .run();
                            assert_eq!(outcome.is_err(), failing);
                        })
                        .join()
                        .unwrap();
                });
                assert_eq!(footprint(), baseline, "failure={failing}, round={round}");
            }
        }
    }
    println!("BYTE_MEMORY_COMPLETED: retained bytes=0, allocations=0");
}
