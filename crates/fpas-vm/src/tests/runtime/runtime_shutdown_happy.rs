//! Successful [`Vm::run`] paths leave global shutdown set for pool teardown.
//!
//! **Documentation:** `docs/pascal/language/concurrency/README.md`,
//! `docs/pascal/language/concurrency/scheduling.md`,
//! `docs/pascal/std/concurrency/task.md`

use crate::Vm;
use fpas_bytecode::{Chunk, Intrinsic, Op, TaskIntrinsic, Value};
use fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN;

use crate::tests::helpers::{build_zero_arg_function_chunk, emit_constant, loc};

// --- Positive: successful runs always shut down the pool ---

#[test]
fn successful_spawn_program_sets_shutdown_before_returning() {
    let callee = "Unit";
    let chunk = build_zero_arg_function_chunk(
        callee,
        |chunk| {
            emit_constant(chunk, Value::function(callee.to_string(), vec![], false));
            chunk.emit(Op::SpawnTask(0), loc());
            chunk.emit(
                Op::Intrinsic(u16::from(Intrinsic::Task(TaskIntrinsic::Wait))),
                loc(),
            );
        },
        |chunk| {
            chunk.emit(Op::Unit, loc());
            chunk.emit(Op::Return, loc());
        },
    );

    let mut vm = Vm::new(chunk);
    vm.run().expect("spawn + wait should succeed");
    assert!(
        vm.is_shutdown_for_tests(),
        "Vm::run must signal shutdown so pool workers can exit"
    );
}

#[test]
fn halt_only_program_without_pool_sets_shutdown() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    assert_eq!(vm.worker_pool_size_for_tests(), 0);
    vm.run().expect("halt-only");
    assert!(vm.is_shutdown_for_tests());
}

#[test]
fn frame_free_main_return_completes_successfully() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Unit, loc());
    chunk.emit(Op::Return, loc());

    let mut vm = Vm::new(chunk);
    vm.run().expect("root Return is a valid program exit");
    assert!(vm.is_shutdown_for_tests());
}

#[test]
fn second_run_on_same_vm_reports_single_use_contract() {
    let mut chunk = Chunk::new();
    chunk.emit(Op::Halt, loc());

    let mut vm = Vm::new(chunk);
    vm.run().expect("first run should succeed");
    let error = vm
        .run()
        .expect_err("second run must fail deterministically");

    assert_eq!(error.code, RUNTIME_VM_SHUTDOWN);
    assert_eq!(error.message, "This VM instance has already been run");
    assert!(
        error
            .help
            .as_deref()
            .is_some_and(|help| help.contains("single-use"))
    );
}

#[test]
fn repeated_fresh_vms_with_spawn_chunk_each_complete_and_set_shutdown() {
    let callee = "Nop";
    for _ in 0..24 {
        let chunk = build_zero_arg_function_chunk(
            callee,
            |chunk| {
                emit_constant(chunk, Value::function(callee.to_string(), vec![], false));
                chunk.emit(Op::SpawnDetachedTask(0), loc());
                chunk.emit(Op::Halt, loc());
            },
            |chunk| {
                chunk.emit(Op::Unit, loc());
                chunk.emit(Op::Return, loc());
            },
        );

        let mut vm = Vm::new(chunk);
        vm.run().expect("repeated detached spawn runs");
        assert!(
            vm.is_shutdown_for_tests(),
            "each run must leave shutdown set for clean pool teardown"
        );
    }
}
