//! Actual allocation balance for byte payloads across cycles, tasks and VM failure.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "isolated allocation regressions use direct assertions"
)]

use fpas_build::{BuildOptions, check_program};
use fpas_runtime::{
    CollectionSchedule, SharedBytes, VerifiedExecutable,
    execution::{
        DebugRunResult, DebugSession, DebugStopReason, DebugTaskState, SourceBreakpoint, Vm,
    },
};
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

fn isolated(case: &str, exercise: fn()) {
    if std::env::var("FPAS_BYTE_MEMORY_CHILD").as_deref() != Ok(case) {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", case, "--nocapture", "--test-threads=1"])
            .env("FPAS_BYTE_MEMORY_CHILD", case)
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
    exercise();
    println!("BYTE_MEMORY_COMPLETED: retained bytes=0, allocations=0");
}

#[test]
fn native_backing_capacity_sharing_and_detachment_are_accounted_once() {
    isolated(
        "native_backing_capacity_sharing_and_detachment_are_accounted_once",
        backing_capacity,
    );
}

fn backing_capacity() {
    const SIZE: usize = 262144;
    let baseline = footprint();
    let empty = SharedBytes::zeroed(0).unwrap();
    let metadata = (footprint().0 - baseline.0, footprint().1 - baseline.1);
    drop(empty);
    assert_eq!(footprint(), baseline);

    for _ in 0..4 {
        let original = SharedBytes::zeroed(SIZE).unwrap();
        let retained = footprint();
        assert_eq!(retained.0 - baseline.0, SIZE as i128 + metadata.0);
        assert_eq!(retained.1 - baseline.1, 1 + metadata.1);
        let mut copy = original.clone();
        let full = original.slice(0, SIZE).unwrap();
        assert_eq!(
            footprint(),
            retained,
            "shared owners must count backing once"
        );
        copy.set_at(0, 255).unwrap();
        assert_eq!(original[0], 0);
        assert_eq!(copy[0], 255);
        assert_eq!(footprint().0 - retained.0, SIZE as i128 + metadata.0);

        let tiny = original.slice(0, 1).unwrap();
        drop(original);
        drop(full);
        assert_eq!(tiny.capacity(), 1);
        assert_eq!(
            footprint().0 - baseline.0,
            SIZE as i128 + 1 + 2 * metadata.0,
            "a proper slice must not retain its parent backing"
        );
        drop(tiny);
        copy.resize(0).unwrap();
        assert_eq!(copy.capacity(), SIZE);
        assert_eq!(
            footprint(),
            retained,
            "clear retains the allocated capacity"
        );
        copy.trim_capacity().unwrap();
        assert_eq!(copy.capacity(), 0);
        assert_eq!(footprint().0 - baseline.0, metadata.0);
        drop(copy);
        assert_eq!(footprint(), baseline);
    }
    println!(
        "BYTE_BACKING: length={SIZE}, capacity={SIZE}, metadata_bytes={}, shared_copy_bytes=0, detached_backing_bytes={SIZE}, trimmed_capacity=0",
        metadata.0
    );
}

#[test]
fn native_bytes_cycles_tasks_and_abandonment_release_all_allocations() {
    isolated(
        "native_bytes_cycles_tasks_and_abandonment_release_all_allocations",
        vm_teardown,
    );
}

fn vm_teardown() {
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
                if !failing {
                    std::thread::scope(|scope| {
                        scope
                            .spawn(|| cancel_blocked_sender(&image, &source, schedule))
                            .join()
                            .unwrap();
                    });
                    assert_eq!(footprint(), baseline, "debug cancellation, round={round}");
                }
            }
        }
    }
}

fn cancel_blocked_sender(image: &VerifiedExecutable, source: &str, schedule: CollectionSchedule) {
    let line = u32::try_from(
        source
            .lines()
            .position(|line| line.trim() == "break;")
            .unwrap()
            + 1,
    )
    .unwrap();
    let mut session = DebugSession::new(image.clone())
        .unwrap()
        .with_collection_schedule(schedule);
    let breakpoint = session
        .set_breakpoint(SourceBreakpoint {
            source: "source-0.fpas".into(),
            line,
            column: None,
        })
        .unwrap();
    assert!(breakpoint.is_verified(), "{breakpoint:?}");
    assert!(matches!(
        session.continue_execution().unwrap(),
        DebugRunResult::Stopped(_)
    ));
    // Hold the parent before scope exit, so the sender must enter its full-channel wait.
    session.pause_task(0).unwrap();
    session.clear_breakpoint(breakpoint.id).unwrap();
    let DebugRunResult::Stopped(stop) = session.continue_execution().unwrap() else {
        panic!("the parent hold must stop after the sender blocks");
    };
    assert_eq!(stop.reason, DebugStopReason::Pause);
    let tasks = session.tasks(0, 16).unwrap();
    let waiting = tasks
        .items
        .iter()
        .filter(|task| task.id != 0 && task.state == DebugTaskState::Waiting)
        .collect::<Vec<_>>();
    assert_eq!(waiting.len(), 1, "{tasks:?}");
    assert!(!waiting[0].paused);
    session.cancel_task(waiting[0].id).unwrap();
    session.resume_task(0).unwrap();
    assert!(matches!(
        session.continue_execution().unwrap(),
        DebugRunResult::Terminated(_)
    ));
}
