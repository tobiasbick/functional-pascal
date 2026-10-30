//! Allocation balance for source-library subviews, including callback cycles.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "isolated memory regressions use direct assertions"
)]

use std::{alloc::System, num::NonZeroUsize, path::Path, process::Command};

use fpas_build::{BuildOptions, check_program};
use fpas_runtime::{CollectionSchedule, execution::Vm};
use stats_alloc::{INSTRUMENTED_SYSTEM, StatsAlloc};

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
fn subview_caches_and_callback_cycles_return_to_the_allocation_baseline() {
    const CASE: &str = "subview_caches_and_callback_cycles_return_to_the_allocation_baseline";
    if std::env::var("FPAS_SUBVIEW_MEMORY_CHILD").as_deref() != Ok(CASE) {
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CASE, "--nocapture", "--test-threads=1"])
            .env("FPAS_SUBVIEW_MEMORY_CHILD", CASE)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("SUBVIEW_MEMORY_COMPLETED"));
        return;
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let entry = root.join("tests/stdlib/tui/subview_lifetime_test.fpas");
    let source = std::fs::read_to_string(&entry).unwrap();
    let library = fpas_project::load_standard_library(&root.join("lib")).unwrap();
    let graph = fpas_project::prepare_program_unit_graph(&[], &Default::default(), Some(&library))
        .unwrap()
        .instantiate(&entry);
    for failing in [false, true] {
        let source = if failing {
            let (body, _) = source.rsplit_once("end.").unwrap();
            format!("{body} Exercise(false); panic('expected subview failure'); end.")
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
            // Initialize Rust thread machinery, without warming up a VM.
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
    println!("SUBVIEW_MEMORY_COMPLETED: 16 sessions; retained bytes=0, allocations=0");
}
