//! Root execution, terminal observation and ordinary shutdown joins.

use super::{Execution, Vm, VmError, diagnostics, tasks, worker::Worker};
use std::sync::Arc;

impl Vm {
    /// Execute the verified root function once.
    ///
    /// # Errors
    ///
    /// Returns the preserved runtime diagnostic for scalar failures, or an internal invariant
    /// diagnostic if a verified executable requests an opcode assigned to a later phase.
    pub fn run(&mut self) -> Result<Execution, VmError> {
        let result = self.run_inner();
        self.hosted.close_process_handles();
        self.hosted.close_file_handles();
        self.hosted.wait_process_handles();
        if let Err(error) = &result {
            self.notify_failure(error);
        }
        if let Some(capture) = self.scheduler.failure_capture.get() {
            capture.finish_catalog();
        }
        result
    }

    fn run_inner(&mut self) -> Result<Execution, VmError> {
        if std::mem::replace(&mut self.has_run, true) {
            return Err(diagnostics::at_address(
                self.executable.executable(),
                fpas_bytecode::InstructionAddress::new(0),
                fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN,
                "This VM instance has already been run",
                "Register VM instances are single-use. Construct a new instance for each run.",
            ));
        }
        let scheduler = Arc::clone(&self.scheduler);
        let mut worker = Worker::for_function_with_state(
            Arc::clone(&self.executable),
            self.executable.executable().entry,
            Vec::new(),
            Arc::clone(&self.globals),
            self.layouts.clone()?,
            Arc::clone(&self.hosted),
        )?
        .with_local_globals(self.use_local_globals)
        .with_scheduler(Some(Arc::clone(&scheduler)));
        if let Some(capture) = scheduler.failure_capture.get() {
            capture.start_root(&mut worker);
        }
        if self.pool_size == 0 {
            let result = worker.run_retained();
            return self.finish_server_execution(result, &worker);
        }
        let pool_size = self.pool_size;
        std::thread::scope(|scope| {
            let mut handles = Vec::with_capacity(pool_size);
            for _ in 0..pool_size {
                let scheduler = Arc::clone(&scheduler);
                let template = worker.pool_template();
                handles.push(scope.spawn(move || tasks::pool::pool_loop(&template, scheduler)));
            }
            let timer_scheduler = Arc::clone(&scheduler);
            let timer = scope.spawn(move || timer_scheduler.timer_loop());
            let root = worker.run_retained();
            let main = self.finish_server_execution(root, &worker);
            if let Err(error) = &main {
                self.notify_failure(error);
                scheduler.fail(error.clone());
            } else {
                scheduler.finish_main();
            }
            self.hosted.shutdown_blocking_operations();
            let mut pool_error = None;
            for handle in handles {
                match handle.join() {
                    Ok(Err(error)) if pool_error.is_none() => pool_error = Some(error),
                    Err(_) if pool_error.is_none() => {
                        pool_error = scheduler.first_error().or_else(|| {
                            Some(diagnostics::internal(
                                self.executable.executable(),
                                fpas_bytecode::InstructionAddress::new(0),
                                "Register task worker panicked",
                            ))
                        });
                    }
                    _ => {}
                }
            }
            if timer.join().is_err() && pool_error.is_none() {
                pool_error = Some(diagnostics::internal(
                    self.executable.executable(),
                    fpas_bytecode::InstructionAddress::new(0),
                    "Register task timer panicked",
                ));
            }
            let result = main.and_then(|value| pool_error.map_or(Ok(value), Err));
            worker.capture_root_globals(result.is_err());
            result
        })
    }

    fn finish_server_execution(
        &self,
        result: Result<Execution, VmError>,
        worker: &Worker,
    ) -> Result<Execution, VmError> {
        worker.capture_root_outcome(&result);
        // An escaping root error already wins over server cleanup and pool joins.
        // Do not wait for those operations before delivering its original state.
        if let Err(error) = &result {
            self.notify_failure(error);
        }
        self.hosted.servers.request_stop_all();
        let result=result.and_then(|execution| {
            self.hosted.servers.ensure_finished().map_err(|message| diagnostics::at_address(
                self.executable.executable(), fpas_bytecode::InstructionAddress::new(0),
                fpas_diagnostics::codes::RUNTIME_VM_SHUTDOWN, message,
                "Use one shutdown path for normal completion and stop requests. Timeout retains ownership."))?;
            Ok(execution)
        });
        if result.is_err() {
            worker.capture_root_outcome(&result);
        }
        result
    }
}
