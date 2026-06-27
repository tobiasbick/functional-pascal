//! [`VmShutdownHandle`] cooperative shutdown from another thread.
//!
//! **Documentation:** `docs/pascal/language/concurrency/scheduling.md`

use crate::Vm;
use fpas_bytecode::{Op, Value};
use fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN;
use std::thread;
use std::time::Duration;

use crate::tests::helpers::{build_zero_arg_function_chunk, emit_constant, loc};

#[test]
fn cooperative_shutdown_aborts_wait_on_spawned_task() {
    let callee = "Spin";
    let chunk = build_zero_arg_function_chunk(
        callee,
        |main| {
            emit_constant(
                main,
                Value::Function {
                    name: callee.to_string(),
                    captures: vec![],
                },
            );
            main.emit(Op::SpawnTask(0), loc());
            main.emit(
                Op::Intrinsic(u16::from(fpas_bytecode::Intrinsic::Task(
                    fpas_bytecode::TaskIntrinsic::Wait,
                ))),
                loc(),
            );
            main.emit(Op::Halt, loc());
        },
        |body| {
            let loop_start = body.len();
            body.emit(Op::Yield, loc());
            body.emit(Op::Jump(loop_start as u32), loc());
        },
    );

    let mut vm = Vm::new(chunk);
    let shutdown = vm.shutdown_handle();
    let handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        shutdown.request_cooperative_shutdown();
    });

    let err = vm.run().expect_err("shutdown should abort wait");
    handle.join().expect("shutdown thread");
    assert_eq!(err.code, RUNTIME_VM_SHUTDOWN);
    assert!(vm.is_shutdown_for_tests());
}
