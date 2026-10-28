//! Hosted `Std.Server` lifetimes. See `docs/pascal/std/network/server.md`.

mod failure;
mod signals;
use failure::Failure;
mod state;
mod watchdog;

use crate::execution::vm::{VmError, worker::Worker};
use crate::{Intrinsic, ServerIntrinsic as Op, SourceLocation, Value};
use state::Lifetime;
use std::collections::HashMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::Duration;

static NEXT: AtomicU64 = AtomicU64::new(0x5352_0000_0000_0001);

/// Bounded lifetime records belonging to one VM; finished records support idempotent completion.
#[derive(Default)]
pub(in crate::execution::vm) struct ServerRegistry {
    pub(in crate::execution::vm) process_authority: AtomicBool,
    entries: Mutex<HashMap<u64, Arc<Lifetime>>>,
}

impl ServerRegistry {
    /// Refuse to report successful execution when the application skipped explicit cleanup.
    pub(in crate::execution::vm) fn ensure_finished(&self) -> Result<(), String> {
        if self
            .entries
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .any(|life| {
                !life
                    .progress
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .finished
            })
        {
            Err("Server shutdown incomplete: close owned groups, handle failures, and call FinishShutdown before returning".into())
        } else {
            Ok(())
        }
    }
    /// Enter the same stop path on normal VM completion or a fatal runtime failure.
    pub(in crate::execution::vm) fn request_stop_all(&self) {
        let entries: Vec<_> = self
            .entries
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .values()
            .cloned()
            .collect();
        for lifetime in entries {
            lifetime.request_stop();
        }
    }
}

#[cfg(test)]
mod tests;

impl Worker {
    /// Dispatch validated lifetime operations without waiting for application work.
    pub(super) fn execute_server_intrinsic(
        &self,
        intrinsic: Intrinsic,
        args: &[Value],
        location: SourceLocation,
    ) -> Result<Option<Option<Value>>, VmError> {
        let Intrinsic::Server(op) = intrinsic else {
            return Ok(None);
        };
        let fail = |message: String| {
            self.runtime_error(
                fpas_diagnostics::codes::RUNTIME_VM_OPERAND_TYPE_MISMATCH,
                message,
                "Use a live Std.Server lifetime belonging to this VM.",
            )
        };
        let result = |value| result(value, self, op, location);
        let value = if op == Op::CreateLifetime {
            let [Value::Integer(grace), Value::Boolean(force)] = args else {
                return Err(fail(
                    "CreateLifetime expects GraceMillis and ForceExit".into(),
                ));
            };
            result(self.create_server(*grace, *force).map(Value::OpaqueHandle))?
        } else {
            let expected = if op == Op::OwnListener { 2 } else { 1 };
            if args.len() != expected {
                return Err(fail(format!(
                    "Server operation expects {expected} arguments"
                )));
            }
            let Value::OpaqueHandle(id) = args[0] else {
                return Err(fail("Expected ServerLifetime".into()));
            };
            let lifetime = self
                .hosted
                .servers
                .entries
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .get(&id)
                .cloned()
                .ok_or_else(|| fail("Server lifetime does not belong to this VM".into()))?;
            match op {
                Op::GetWorkGroup => Value::OpaqueHandle(lifetime.group),
                Op::GetStopToken => Value::OpaqueHandle(lifetime.cancellation.token()),
                Op::IsReady => Value::Boolean(lifetime.ready()),
                Op::RequestStop => Value::Boolean(lifetime.request_stop()),
                Op::RemainingMillis => Value::Integer(lifetime.remaining()),
                Op::FinishShutdown => result(lifetime.finish(self.task_id).map(Value::Boolean))?,
                Op::ShutdownErrors => Value::Array(
                    lifetime
                        .errors()
                        .into_iter()
                        .map(|(id, error)| {
                            let value = error.into_value(
                                self.layouts.as_ref(),
                                "Std.Net.CloseListener",
                                &[id.to_string()],
                                location,
                            )?;
                            Ok(Value::Str(value.to_string().into()))
                        })
                        .collect::<Result<Vec<_>, VmError>>()?
                        .into(),
                ),
                Op::ObserveSignals => result(
                    if !self
                        .hosted
                        .servers
                        .process_authority
                        .load(Ordering::Acquire)
                        || !lifetime.ready()
                    {
                        Err(Failure::State)
                    } else {
                        signals::observe(&lifetime).map(Value::Boolean)
                    },
                )?,
                Op::OwnListener => {
                    let Value::OpaqueHandle(listener) = args[1] else {
                        return Err(fail("Expected Std.Net.Listener".into()));
                    };
                    let entries = self
                        .hosted
                        .servers
                        .entries
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    if entries
                        .iter()
                        .any(|(other, life)| *other != id && life.owns_listener(listener))
                    {
                        result(Err(Failure::State))?
                    } else {
                        result(lifetime.own_listener(listener).map(Value::Boolean))?
                    }
                }
                Op::CreateLifetime => unreachable!(),
            }
        };
        Ok(Some(Some(value)))
    }

    fn create_server(&self, grace: i64, force: bool) -> Result<u64, Failure> {
        if !(0..=86_400_000).contains(&grace) {
            return Err(Failure::Argument(
                "GraceMillis",
                "GraceMillis must be between 0 and 86400000".into(),
            ));
        }
        if force
            && !self
                .hosted
                .servers
                .process_authority
                .load(Ordering::Acquire)
        {
            return Err(Failure::State);
        }
        if self.debug_tasks && force {
            return Err(Failure::State);
        }
        let mut entries = self
            .hosted
            .servers
            .entries
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if entries.len() >= 256 {
            return Err(Failure::State);
        }
        let scheduler = self.scheduler_ref().map_err(Failure::Invariant)?;
        let group = scheduler
            .groups
            .create(self.task_id, || self.hosted.cancellations.create_owned())
            .map_err(|error| Failure::Invariant(self.group_error(error)))?;
        let lifetime = Arc::new(Lifetime::new(
            group,
            self.task_id,
            self.hosted.cancellations.create_owned(),
            Duration::from_millis(grace as u64),
            scheduler,
            &self.hosted,
        ));
        if force && let Err(error) = watchdog::start(&lifetime) {
            scheduler
                .groups
                .begin_close(group, self.task_id)
                .map_err(|error| Failure::Invariant(self.group_error(error)))?;
            scheduler
                .groups
                .take_closed(group)
                .map_err(|error| Failure::Invariant(self.group_error(error)))?;
            return Err(error);
        }
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        entries.insert(id, lifetime);
        Ok(id)
    }
}

fn result(
    value: Result<Value, Failure>,
    worker: &Worker,
    operation: Op,
    location: SourceLocation,
) -> Result<Value, VmError> {
    match value {
        Ok(value) => Ok(Value::result_ok(value)),
        Err(error) => error_value(error, worker, operation, location),
    }
}
#[cold]
#[inline(never)]
fn error_value(
    error: Failure,
    worker: &Worker,
    operation: Op,
    location: SourceLocation,
) -> Result<Value, VmError> {
    Ok(Value::result_error(error.into_value(
        worker,
        &Intrinsic::Server(operation).debugger_name(),
        location,
    )?))
}
