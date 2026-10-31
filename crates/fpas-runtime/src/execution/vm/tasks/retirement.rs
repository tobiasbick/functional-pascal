//! Cooperative retirement of native operations abandoned by cancellation.

use std::time::Duration;

use super::TaskSuspension;
use crate::execution::vm::{VmError, shared::wakeups::WakeSignal, worker::Worker};

pub(in crate::execution::vm) enum NativeRetirement {
    Acp(crate::execution::vm::hosted::acp::wait::Retirement),
    Mcp(crate::execution::vm::hosted::mcp::wait::Retirement),
}

impl NativeRetirement {
    fn poll(&self, worker: &Worker) -> Result<bool, VmError> {
        match self {
            Self::Acp(wait) => wait.poll(worker),
            Self::Mcp(wait) => wait.poll(worker),
        }
    }
}

impl Worker {
    pub(in crate::execution::vm) fn retire_cancelled_suspension(
        &mut self,
        suspension: TaskSuspension,
        error: VmError,
    ) -> Result<bool, VmError> {
        let wait = match suspension {
            TaskSuspension::Acp(wait) => wait.into_retirement().map(NativeRetirement::Acp),
            TaskSuspension::Mcp(wait) => wait.into_retirement().map(NativeRetirement::Mcp),
            TaskSuspension::NativeRetirement { wait, error } => {
                return self.poll_native_retirement(*wait, error);
            }
            _ => None,
        };
        let Some(wait) = wait else {
            self.suspend_requested = false;
            return Err(error);
        };
        if self.task_id == 0 && !self.debug_tasks {
            loop {
                let signal = WakeSignal::new();
                let _registration = self
                    .scheduler
                    .as_ref()
                    .map(|scheduler| scheduler.subscribe(&signal));
                match wait.poll(self) {
                    Ok(false) => {}
                    Ok(true) => {
                        self.suspend_requested = false;
                        return Err(error);
                    }
                    Err(failure) => {
                        self.suspend_requested = false;
                        return Err(failure);
                    }
                }
                let _native_wait = self.native_wait(Vec::new())?;
                signal.wait(Duration::from_millis(10));
            }
        }
        self.poll_native_retirement(wait, error)
    }

    pub(in crate::execution::vm) fn poll_native_retirement(
        &mut self,
        wait: NativeRetirement,
        error: VmError,
    ) -> Result<bool, VmError> {
        let complete = match wait.poll(self) {
            Ok(complete) => complete,
            Err(failure) => {
                self.suspend_requested = false;
                return Err(failure);
            }
        };
        if complete {
            self.suspend_requested = false;
            Err(error)
        } else {
            self.task_suspension = Some(TaskSuspension::NativeRetirement {
                wait: Box::new(wait),
                error,
            });
            self.suspend_requested = true;
            Ok(false)
        }
    }

    pub(in crate::execution::vm) fn retire_wait_error(
        &mut self,
        error: VmError,
    ) -> Result<(), VmError> {
        let Some(suspension) = self.task_suspension.take() else {
            return Err(error);
        };
        self.retire_cancelled_suspension(suspension, error)
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::vm::TaskResultPoll;
    use crate::execution::vm::tasks::{TaskScheduler, pool};
    use std::{
        io::{Read, Write},
        net::TcpStream,
        sync::{Arc, mpsc},
        time::{Duration, Instant},
    };

    #[test]
    fn cancelled_mcp_close_waits_for_the_socket_cleanup_receipt() {
        let runtime = fpas_mcp::HttpRuntime::new().expect("HTTP runtime");
        let server = fpas_mcp::Server::new(fpas_mcp::Catalog::new(Default::default()).unwrap());
        let options = fpas_mcp::HttpOptions {
            connections: 1,
            ..Default::default()
        };
        let mut ticket = runtime.publish(server, options).expect("publish");
        let deadline = Instant::now() + Duration::from_secs(5);
        let publication = loop {
            if let Some(result) = ticket.try_complete() {
                break result.expect("publication");
            }
            assert!(Instant::now() < deadline, "publication must start");
            std::thread::yield_now();
        };
        let url = publication.url();
        let address = url
            .strip_prefix("http://")
            .unwrap()
            .split('/')
            .next()
            .unwrap();
        let mut held = TcpStream::connect(address).expect("first connection");
        held.write_all(b"POST /mcp HTTP/1.1\r\n").unwrap();
        // With one allowed connection, rejection of a second proves the first is admitted.
        let mut probe = TcpStream::connect(address).expect("second connection");
        probe
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let _ = probe.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
        match probe.read(&mut [0]) {
            Ok(0) => {}
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::ConnectionReset
                        | std::io::ErrorKind::ConnectionAborted
                        | std::io::ErrorKind::BrokenPipe
                ) => {}
            other => panic!("second socket was not rejected: {other:?}"),
        }
        let receipt = publication.receipt();
        publication.stop();
        assert!(
            receipt.result().is_none(),
            "physical cleanup waits for the held socket"
        );

        let (program, errors) = fpas_parser::parse("program T; begin null; end.");
        assert!(errors.is_empty());
        let image = Arc::new(fpas_compiler::compile(&program).expect("fixture"));
        let scheduler = Arc::new(TaskScheduler::new());
        let mut worker = Worker::new(image)
            .unwrap()
            .with_scheduler(Some(scheduler.clone()));
        worker.task_id = 7;
        worker.retain_result = true;
        worker.task_suspension = Some(TaskSuspension::Mcp(Box::new(
            crate::execution::vm::hosted::mcp::wait::Wait::Close {
                receipts: vec![receipt],
                id: None,
                endpoint: false,
                release: false,
                first: true,
                destination: None,
            },
        )));
        scheduler.register_result(7);
        let template = worker.pool_template();
        scheduler.enqueue(worker.take_task_state());
        scheduler.request_cancel();
        let pool_scheduler = scheduler.clone();
        let (finished, completion) = mpsc::channel();
        let pool = std::thread::spawn(move || {
            let result = pool::pool_loop(&template, pool_scheduler);
            finished.send(result).unwrap();
        });
        let timer_scheduler = scheduler.clone();
        let timer = std::thread::spawn(move || timer_scheduler.timer_loop());
        let premature = completion.recv_timeout(Duration::from_millis(100)).is_ok();
        let pending = matches!(scheduler.poll_result(7), TaskResultPoll::Pending);
        drop(held);
        let completed = completion.recv_timeout(Duration::from_secs(10));
        pool.join().unwrap();
        timer.join().unwrap();
        runtime.request_shutdown();
        runtime.wait_closed();
        assert!(
            !premature && pending,
            "task completion must wait for the MCP receipt"
        );
        completed
            .expect("retirement completes")
            .expect("cancellation is contained");
        assert!(matches!(
            scheduler.poll_result(7),
            TaskResultPoll::Failed(_)
        ));
    }
}
