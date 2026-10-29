//! Bounded cooperative pipe work followed by one owned completion path.

use super::{Execution, Failure, Kind, Output};
use std::io;
use std::process::ExitStatus;
use std::task::Poll;
use std::time::Instant;

// Each pipe probe transfers at most 8 KiB per stream. Bound work per VM turn.
const PROBES_PER_TURN: usize = 8;

impl Execution {
    pub(in crate::execution::vm::hosted::proc) fn made_progress(&self) -> bool {
        self.progress
    }

    pub(in crate::execution::vm::hosted::proc) fn poll(
        &mut self,
        input: &[u8],
        mut cancelled: impl FnMut() -> bool,
    ) -> Poll<Result<Output, Failure>> {
        for _ in 0..PROBES_PER_TURN {
            match self.step(input, &mut cancelled) {
                Ok(Some(code)) => return Poll::Ready(self.finish(Ok(code))),
                Err(kind) => return Poll::Ready(self.finish(Err(kind))),
                Ok(None) if !self.progress => return Poll::Pending,
                Ok(None) => {}
            }
        }
        Poll::Pending
    }

    fn step(
        &mut self,
        input: &[u8],
        cancelled: &mut dyn FnMut() -> bool,
    ) -> Result<Option<i64>, Kind> {
        let Some(reservation) = &self.reservation else {
            return Err(Kind::Execution(io::Error::other(
                "Process call already completed",
            )));
        };
        let process = &reservation.process;
        let deadline = self.deadline;
        let check = |cancelled: &mut dyn FnMut() -> bool| {
            if process.stopped() || cancelled() {
                Err(Kind::Cancelled)
            } else if deadline.is_some_and(|end| Instant::now() >= end) {
                Err(Kind::Timeout)
            } else {
                Ok(())
            }
        };
        check(cancelled)?;
        let status = process.poll().map_err(Kind::Wait)?;
        check(cancelled)?;
        if let Some(status) = status
            && self.pipes.complete()
        {
            return exit_code(status).map(Some);
        }
        self.progress = self.pipes.poll(input)?;
        let status = process.poll().map_err(Kind::Wait)?;
        check(cancelled)?;
        if let Some(status) = status
            && self.pipes.complete()
        {
            return exit_code(status).map(Some);
        }
        Ok(None)
    }

    fn finish(&mut self, outcome: Result<i64, Kind>) -> Result<Output, Failure> {
        let Some(reservation) = self.reservation.take() else {
            return Err(Failure::new(Kind::Execution(io::Error::other(
                "Process call already completed",
            ))));
        };
        let terminated = reservation.process.terminate();
        if outcome.is_err() {
            self.pipes.drain_after_abort();
        }
        self.pipes.close();
        let reaped = reservation.process.reap();
        let (stdout, stderr) = self.pipes.output();
        let outcome = outcome
            .and_then(|code| terminated.map_err(Kind::Execution).map(|()| code))
            .and_then(|code| reaped.map_err(Kind::Wait).map(|_| code));
        // Options calls deliver stderr only after the owned group and pipes finish.
        if let Some(receiver) = self.deferred.take() {
            for line in stderr.split_inclusive('\n') {
                receiver(line.trim_end_matches('\n').trim_end_matches('\r'));
            }
        }
        match outcome {
            Ok(code) => Ok(Output {
                code,
                stdout,
                stderr,
            }),
            Err(kind) => Err(Failure {
                kind,
                stdout,
                stderr,
            }),
        }
    }
}

fn exit_code(status: ExitStatus) -> Result<i64, Kind> {
    status
        .code()
        .map(i64::from)
        .ok_or_else(|| Kind::Wait(io::Error::other("Process terminated without an exit code")))
}
