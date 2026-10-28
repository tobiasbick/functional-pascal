//! Ordinary scheduler observations, original storage ownership, and bounded native waits.

mod bounds;
mod lifecycle;
mod live_peers;
mod native;
mod regressions;

use super::{observe, scalar, vm};
use crate::execution::vm::{Vm, worker::Worker};
use crate::{FunctionId, Value};
use std::{sync::Arc, thread::JoinHandle};

fn worker(vm: &Vm, name: &str, arguments: Vec<Value>) -> Worker {
    let image = vm.executable.executable();
    let target = image
        .functions
        .iter()
        .position(|function| {
            image
                .strings
                .get(function.name)
                .unwrap()
                .eq_ignore_ascii_case(name)
        })
        .unwrap();
    Worker::for_function_with_state(
        Arc::clone(&vm.executable),
        FunctionId::try_from_index(target).unwrap(),
        arguments,
        Arc::clone(&vm.globals),
        vm.layouts.clone().unwrap(),
        Arc::clone(&vm.hosted),
    )
    .unwrap()
    .with_scheduler(Some(Arc::clone(&vm.scheduler)))
}

fn enqueue(vm: &Vm, name: &str, arguments: Vec<Value>) -> u64 {
    let mut worker = worker(vm, name, arguments);
    worker.task_id = vm.scheduler.alloc_id();
    worker.retain_result = true;
    let id = worker.task_id;
    vm.scheduler.register_result(id);
    vm.scheduler.enqueue(worker.take_task_state());
    id
}

struct PoolGuard {
    shutdown: crate::execution::vm::ShutdownHandle,
    thread: Option<JoinHandle<()>>,
}
impl Drop for PoolGuard {
    fn drop(&mut self) {
        self.shutdown.shutdown();
        if let Some(thread) = self.thread.take() {
            let result = thread.join();
            if !std::thread::panicking() {
                result.unwrap();
            }
        }
    }
}

fn start_pool(vm: &Vm) -> PoolGuard {
    let template = worker(vm, "Barrier", vec![]).pool_template();
    let scheduler = Arc::clone(&vm.scheduler);
    let thread = std::thread::spawn(move || {
        if let Err(error) =
            crate::execution::vm::tasks::pool::pool_loop(&template, Arc::clone(&scheduler))
        {
            assert!(scheduler.is_aborted(), "unexpected pool error: {error:?}");
        }
    });
    PoolGuard {
        shutdown: vm.shutdown_handle(),
        thread: Some(thread),
    }
}

fn start_timer(vm: &Vm) -> PoolGuard {
    let scheduler = Arc::clone(&vm.scheduler);
    let thread = std::thread::spawn(move || scheduler.timer_loop());
    PoolGuard {
        shutdown: vm.shutdown_handle(),
        thread: Some(thread),
    }
}
