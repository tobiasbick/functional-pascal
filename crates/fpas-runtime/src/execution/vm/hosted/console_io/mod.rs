//! Owned console requests shared by synchronous execution and suspended task jobs.

mod execute;
mod order;
mod request;
mod testing;
#[cfg(test)]
mod tests;
pub(in crate::execution::vm) mod wait;
use super::HostedState;
use crate::{
    SourceLocation,
    stdlib::{Console, StdError},
};
pub(super) use order::Order;
pub(in crate::execution::vm) use request::supported;
use std::{
    sync::{Mutex, MutexGuard, TryLockError},
    time::Duration,
};

enum Command {
    Console(request::Call),
    Test(testing::Call),
}
struct Request {
    command: Command,
    ticket: order::Ticket,
}
impl Request {
    fn prepare(
        worker: &crate::execution::vm::worker::Worker,
        operation: crate::Intrinsic,
        values: &[crate::Value],
        location: SourceLocation,
    ) -> Result<Self, crate::execution::vm::VmError> {
        let (command, lanes) = match operation {
            crate::Intrinsic::Console(op) => {
                let call = request::Call::prepare(worker, op, values, location)?;
                let lanes = call.lanes();
                (Command::Console(call), lanes)
            }
            crate::Intrinsic::Test(op) => {
                let call = testing::Call::prepare(worker, op, values, location)?;
                let lanes = call.lanes();
                (Command::Test(call), lanes)
            }
            _ => unreachable!("native console request"),
        };
        Ok(Self {
            command,
            ticket: worker.hosted.console_order.ticket(lanes, location)?,
        })
    }
    fn ready(&self) -> bool {
        self.ticket.ready()
    }
    fn location(&self) -> SourceLocation {
        match &self.command {
            Command::Console(call) => call.location,
            Command::Test(call) => call.location(),
        }
    }
    fn run(
        &self,
        state: &HostedState,
        cancelled: impl Fn() -> bool,
    ) -> Result<request::Output, StdError> {
        while !self.ready() {
            if cancelled() {
                return Err(self::cancelled(self.location()));
            }
            std::thread::park_timeout(Duration::from_millis(1));
        }
        match &self.command {
            Command::Console(call) => call.run(state, cancelled),
            Command::Test(call) => call.run(state, cancelled),
        }
    }
}
pub(in crate::execution::vm) fn supported_intrinsic(intrinsic: crate::Intrinsic) -> bool {
    match intrinsic {
        crate::Intrinsic::Console(op) => supported(op),
        crate::Intrinsic::Test(op) => testing::supported(op),
        _ => false,
    }
}
pub(super) fn supported_test(op: crate::TestIntrinsic) -> bool {
    testing::supported(op)
}

fn cancelled(location: SourceLocation) -> StdError {
    crate::stdlib::error::std_runtime_error(
        fpas_diagnostics::codes::RUNTIME_TASK_CANCELLED,
        "Console operation was cancelled",
        "Start a new operation if needed.",
        location,
    )
}

fn lock<'a, T>(
    mutex: &'a Mutex<T>,
    stop: impl Fn() -> bool,
    location: SourceLocation,
) -> Result<MutexGuard<'a, T>, StdError> {
    loop {
        if stop() {
            return Err(cancelled(location));
        }
        match mutex.try_lock() {
            Ok(guard) => return Ok(guard),
            Err(TryLockError::Poisoned(error)) => return Ok(error.into_inner()),
            Err(TryLockError::WouldBlock) => std::thread::park_timeout(Duration::from_millis(1)),
        }
    }
}

fn discard_regions(state: &HostedState, console: &mut Console) {
    let regions = std::mem::take(
        &mut *state
            .console_regions
            .lock()
            .unwrap_or_else(|p| p.into_inner()),
    );
    for id in regions {
        let _ = console.discard_region(id, SourceLocation::new(1, 1));
    }
}

pub(super) fn retire_regions(state: &HostedState) {
    let mut console = state.console.lock().unwrap_or_else(|p| p.into_inner());
    discard_regions(state, &mut console);
}

impl crate::execution::vm::worker::Worker {
    pub(super) fn execute_test_console_owned(
        &self,
        operation: crate::TestIntrinsic,
        values: &[crate::Value],
        location: SourceLocation,
    ) -> Result<Option<crate::Value>, crate::execution::vm::VmError> {
        self.execute_ordered_console(crate::Intrinsic::Test(operation), values, location)
    }
    pub(super) fn execute_console_owned(
        &self,
        operation: crate::ConsoleIntrinsic,
        values: &[crate::Value],
        location: SourceLocation,
    ) -> Result<Option<crate::Value>, crate::execution::vm::VmError> {
        self.execute_ordered_console(crate::Intrinsic::Console(operation), values, location)
    }
    fn execute_ordered_console(
        &self,
        operation: crate::Intrinsic,
        values: &[crate::Value],
        location: SourceLocation,
    ) -> Result<Option<crate::Value>, crate::execution::vm::VmError> {
        let request = Request::prepare(self, operation, values, location)?;
        let output = request.run(&self.hosted, || false);
        drop(request);
        output?.into_value(self, location)
    }
}

impl crate::execution::vm::Vm {
    /// Install an owned text reader before execution; queued input keeps precedence.
    /// In-flight host reads must return before shutdown can finish joining their job.
    pub fn set_text_input_reader(&mut self, reader: impl std::io::BufRead + Send + 'static) {
        self.hosted
            .text_input
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .set_reader(reader);
    }
}
