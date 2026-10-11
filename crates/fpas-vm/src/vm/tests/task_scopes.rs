//! Cancellation at instruction and synchronous callback boundaries.

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use fpas_bytecode::{FunctionId, Value};
use fpas_diagnostics::codes::{RUNTIME_PROGRAM_PANIC, RUNTIME_TASK_CANCELLED};

use crate::vm::{VmError, hosted::HostedState, tasks::TaskScheduler, worker::Worker};

mod blocking_waits;

enum Trigger {
    Cancel(u64),
    Fail(u64, VmError),
}

struct ScopeWriter {
    scheduler: Arc<TaskScheduler>,
    trigger: Option<Trigger>,
    output: Arc<Mutex<Vec<u8>>>,
}

impl Write for ScopeWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.output.lock().expect("output").extend_from_slice(bytes);
        match self.trigger.take() {
            Some(Trigger::Cancel(scope)) => self.scheduler.scopes.cancel(scope),
            Some(Trigger::Fail(task, error)) => {
                self.scheduler.store_failure(task, error);
            }
            None => {}
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn worker() -> Worker {
    let source = "program Cancellation;
uses Std.Console;
function Warm(): integer;
begin return 0; end function;
function Work(): integer;
begin WriteLn('first'); WriteLn('after'); return 1; end function;
begin WriteLn('first'); WriteLn('after'); end.";
    let (program, errors) = fpas_parser::parse(source);
    assert!(errors.is_empty(), "{errors:?}");
    Worker::new(Arc::new(fpas_compiler::compile(&program).expect("compile")))
        .expect("worker")
        .with_scheduler(Some(Arc::new(TaskScheduler::new())))
}

fn callback(worker: &Worker, name: &str) -> Value {
    let image = worker.executable.executable();
    let index = image
        .functions
        .iter()
        .position(|info| {
            image
                .strings
                .get(info.name)
                .is_some_and(|value| value.eq_ignore_ascii_case(name))
        })
        .expect("callback target");
    Value::function(
        FunctionId::new(index.try_into().expect("function index")),
        name,
        Vec::new(),
    )
}

fn install_writer(worker: &mut Worker, trigger: Trigger) -> Arc<Mutex<Vec<u8>>> {
    let output = Arc::new(Mutex::new(Vec::new()));
    let console = fpas_std::Console::with_writer(Box::new(ScopeWriter {
        scheduler: Arc::clone(worker.scheduler.as_ref().expect("scheduler")),
        trigger: Some(trigger),
        output: Arc::clone(&output),
    }));
    worker.hosted = Arc::new(HostedState::new(console, Vec::new()));
    output
}

fn assert_stopped(error: VmError, output: &Arc<Mutex<Vec<u8>>>) {
    assert_eq!(error.code, RUNTIME_TASK_CANCELLED, "{error:?}");
    let bytes = output.lock().expect("output");
    let text = std::str::from_utf8(&bytes).expect("UTF-8 output");
    assert!(
        text.contains("first"),
        "cancellation happens during the first instruction"
    );
    assert!(
        !text.contains("after"),
        "no later instruction may run: {text}"
    );
}

#[test]
fn cancelled_children_stop_before_the_next_instruction_in_a_batch() {
    let mut worker = worker();
    let scheduler = Arc::clone(worker.scheduler.as_ref().expect("scheduler"));
    let (scope, _) = scheduler.scopes.open();
    worker.task_id = 1;
    worker.owner_scope = scheduler.scopes.enroll(scope, worker.task_id);
    let output = install_writer(&mut worker, Trigger::Cancel(scope));
    assert_stopped(worker.run_task().expect_err("cancelled child"), &output);
}

#[test]
fn synchronous_callbacks_observe_enclosing_scope_failure_with_fresh_and_cached_workers() {
    for cached in [false, true] {
        let mut worker = worker();
        worker.enter_task_scope().expect("scope");
        let scheduler = Arc::clone(worker.scheduler.as_ref().expect("scheduler"));
        let scope = worker.spawning_scope().expect("scope identity");
        scheduler.scopes.enroll(scope, 1).expect("child");
        scheduler.register_result(1);
        let failure = worker.runtime_error(RUNTIME_PROGRAM_PANIC, "child failed", "scope failure");
        let output = install_writer(&mut worker, Trigger::Fail(1, failure));
        if cached {
            worker
                .call_callback_sync(&callback(&worker, "Warm"), [])
                .expect("warm cache");
        }
        let error = worker
            .call_callback_sync(&callback(&worker, "Work"), [])
            .expect_err("callback must stop when a child of its caller fails");
        assert_stopped(error, &output);
        let error =
            worker.runtime_error(RUNTIME_TASK_CANCELLED, "stopped callback", "scope failure");
        let outcome = worker.unwind_task_scopes_blocking(error);
        assert_eq!(outcome.code, RUNTIME_PROGRAM_PANIC);
        assert_eq!(
            worker
                .call_callback_sync(&callback(&worker, "Warm"), [])
                .expect("reuse after scope exit"),
            Value::Integer(0)
        );
    }
}
