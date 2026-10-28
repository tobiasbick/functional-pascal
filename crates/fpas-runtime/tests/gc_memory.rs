//! Process-isolated, cross-thread allocation balances for runtime reclamation.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "memory regression fixtures fail with direct, local assertions"
)]

#[path = "gc_memory/graphs.rs"]
mod graphs;
#[path = "gc_memory/measurement.rs"]
mod measurement;
#[path = "gc_memory/sessions.rs"]
mod sessions;
#[path = "gc_memory/workloads.rs"]
mod workloads;

use stats_alloc::{INSTRUMENTED_SYSTEM, StatsAlloc};
use std::alloc::System;

#[global_allocator]
static ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

#[test]
fn allocator_observes_retained_bytes_and_cross_thread_release() {
    measurement::isolated(
        "allocator_observes_retained_bytes_and_cross_thread_release",
        measurement::verify_counter,
    );
}

#[test]
fn collected_aggregate_payloads_return_to_the_allocation_baseline() {
    measurement::isolated(
        "collected_aggregate_payloads_return_to_the_allocation_baseline",
        graphs::collect_payloads,
    );
}

#[test]
fn vm_cycles_return_to_the_allocation_baseline() {
    measurement::isolated("vm_cycles_return_to_the_allocation_baseline", workloads::vm);
}

#[test]
fn task_cycles_return_to_the_allocation_baseline() {
    measurement::isolated(
        "task_cycles_return_to_the_allocation_baseline",
        workloads::tasks,
    );
}

#[test]
fn callback_cycles_keep_peak_memory_bounded() {
    measurement::isolated(
        "callback_cycles_keep_peak_memory_bounded",
        workloads::callback_cycles,
    );
}

#[test]
fn callback_exports_release_payloads_without_session_growth() {
    measurement::isolated(
        "callback_exports_release_payloads_without_session_growth",
        sessions::callbacks,
    );
}

#[test]
fn debugger_inspection_and_evaluation_release_payloads() {
    measurement::isolated(
        "debugger_inspection_and_evaluation_release_payloads",
        sessions::debugger,
    );
}
