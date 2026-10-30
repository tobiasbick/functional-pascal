//! Process-isolated, cross-thread allocation balances for runtime reclamation.
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "memory regression fixtures fail with direct, local assertions"
)]

#[path = "gc_memory/debug_edits.rs"]
mod debug_edits;
#[path = "gc_memory/files.rs"]
mod files;
#[path = "gc_memory/graphs.rs"]
mod graphs;
#[path = "gc_memory/measurement.rs"]
mod measurement;
#[path = "gc_memory/mixed.rs"]
mod mixed;
#[path = "gc_memory/native_acp.rs"]
mod native_acp;
#[path = "gc_memory/native_console.rs"]
mod native_console;
#[path = "gc_memory/native_entropy.rs"]
mod native_entropy;
#[path = "gc_memory/native_process.rs"]
mod native_process;
#[path = "gc_memory/network.rs"]
mod network;
#[path = "gc_memory/processes.rs"]
mod processes;
#[path = "gc_memory/random_graph/mod.rs"]
mod random_graph;
#[path = "gc_memory/sessions.rs"]
mod sessions;
#[path = "gc_memory/shutdown.rs"]
mod shutdown;
#[path = "gc_memory/tokenizer.rs"]
mod tokenizer;
#[path = "gc_memory/workloads.rs"]
mod workloads;

use stats_alloc::{INSTRUMENTED_SYSTEM, StatsAlloc};
use std::alloc::System;

#[global_allocator]
static ALLOCATOR: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

#[test]
fn tokenizer_tables_and_callbacks_return_to_the_allocation_baseline() {
    measurement::isolated(
        "tokenizer_tables_and_callbacks_return_to_the_allocation_baseline",
        tokenizer::teardown,
    );
}

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

#[test]
fn mixed_task_waits_keep_live_memory_bounded() {
    measurement::isolated(
        "mixed_task_waits_keep_live_memory_bounded",
        mixed::live_mixed_waits,
    );
}

#[test]
fn mixed_task_waits_release_memory_after_completion_and_failure() {
    measurement::isolated(
        "mixed_task_waits_release_memory_after_completion_and_failure",
        mixed::mixed_wait_teardown,
    );
}

#[test]
fn debugger_frame_restarts_release_discarded_payloads() {
    measurement::isolated(
        "debugger_frame_restarts_release_discarded_payloads",
        debug_edits::restarts,
    );
}

#[test]
fn debugger_forced_returns_release_payloads_and_failed_preparation() {
    measurement::isolated(
        "debugger_forced_returns_release_payloads_and_failed_preparation",
        debug_edits::forced_returns,
    );
}

#[test]
fn debugger_mutations_release_payloads_and_failed_preparation() {
    measurement::isolated(
        "debugger_mutations_release_payloads_and_failed_preparation",
        debug_edits::mutations,
    );
}

#[test]
fn pending_tasks_release_memory_after_explicit_vm_shutdown() {
    measurement::isolated(
        "pending_tasks_release_memory_after_explicit_vm_shutdown",
        shutdown::explicit_shutdown,
    );
}

#[test]
fn pending_tasks_release_memory_after_root_failure() {
    measurement::isolated(
        "pending_tasks_release_memory_after_root_failure",
        shutdown::runtime_failure,
    );
}

#[test]
fn pending_tasks_release_memory_after_scope_exit() {
    measurement::isolated(
        "pending_tasks_release_memory_after_scope_exit",
        shutdown::scope_exit,
    );
}

#[test]
fn seeded_graph_roots_and_mutations_match_the_reference_model() {
    measurement::isolated(
        "seeded_graph_roots_and_mutations_match_the_reference_model",
        random_graph::generated_graphs,
    );
}

#[test]
fn native_socket_waits_release_frames_cycles_and_jobs() {
    measurement::isolated(
        "native_socket_waits_release_frames_cycles_and_jobs",
        network::socket_wait_teardown,
    );
}

#[test]
fn native_connects_and_dns_release_frames_and_jobs() {
    measurement::isolated(
        "native_connects_and_dns_release_frames_and_jobs",
        network::socket_connect_completion,
    );
}

#[test]
fn native_socket_configuration_releases_callbacks_and_jobs() {
    measurement::isolated(
        "native_socket_configuration_releases_callbacks_and_jobs",
        network::socket_configuration_teardown,
    );
}

#[test]
fn owned_process_waits_release_frames_cycles_pipes_and_children() {
    measurement::isolated(
        "owned_process_waits_release_frames_cycles_pipes_and_children",
        processes::owned_process_teardown,
    );
}

#[test]
fn native_process_boundaries_release_frames_and_jobs() {
    measurement::isolated(
        "native_process_boundaries_release_frames_and_jobs",
        native_process::teardown,
    );
}

#[test]
fn native_console_boundaries_release_frames_and_jobs() {
    measurement::isolated(
        "native_console_boundaries_release_frames_and_jobs",
        native_console::teardown,
    );
}

#[test]
fn native_entropy_releases_callbacks_seeds_and_buffers() {
    measurement::isolated(
        "native_entropy_releases_callbacks_seeds_and_buffers",
        native_entropy::entropy,
    );
}

#[test]
fn initial_mcp_executors_release_callbacks_and_publications() {
    measurement::isolated(
        "initial_mcp_executors_release_callbacks_and_publications",
        native_entropy::mcp,
    );
}

#[test]
fn native_acp_boundaries_release_frames_and_jobs() {
    measurement::isolated(
        "native_acp_boundaries_release_frames_and_jobs",
        native_acp::teardown,
    );
}

#[cfg(unix)]
#[test]
fn native_acp_file_jobs_release_frames_and_payloads() {
    measurement::isolated(
        "native_acp_file_jobs_release_frames_and_payloads",
        native_acp::file_jobs,
    );
}

#[test]
#[ignore = "owned subprocess fixture, launched by process allocation regressions"]
fn process_child() {
    processes::child::run();
}

#[test]
fn selectable_process_pipes_release_callbacks_and_native_payloads() {
    measurement::isolated(
        "selectable_process_pipes_release_callbacks_and_native_payloads",
        processes::selectable_pipe_teardown,
    );
}

#[test]
fn native_file_jobs_release_frames_cycles_and_payloads() {
    measurement::isolated(
        "native_file_jobs_release_frames_cycles_and_payloads",
        files::file_job_teardown,
    );
}

#[test]
fn native_path_jobs_release_frames_cycles_and_results() {
    measurement::isolated(
        "native_path_jobs_release_frames_cycles_and_results",
        files::path_job_teardown,
    );
}
