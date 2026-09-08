//! Hosted `Std.Server` lifetimes. See `docs/pascal/std/network/server.md`.

mod signals;
mod state;
mod watchdog;

use crate::vm::{VmError, worker::Worker};
use fpas_bytecode::{Intrinsic, ServerIntrinsic as Op, SourceLocation, Value};
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
pub(in crate::vm) struct ServerRegistry {
    pub(in crate::vm) process_authority: AtomicBool,
    entries: Mutex<HashMap<u64, Arc<Lifetime>>>,
}

impl ServerRegistry {
    /// Refuse to report successful execution when the application skipped explicit cleanup.
    pub(in crate::vm) fn ensure_finished(&self) -> Result<(), String> {
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
    pub(in crate::vm) fn request_stop_all(&self) {
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
        _location: SourceLocation,
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
        let value = if op == Op::CreateLifetime {
            let [Value::Integer(grace), Value::Boolean(force)] = args else {
                return Err(fail(
                    "CreateLifetime expects GraceMillis and ForceExit".into(),
                ));
            };
            result(self.create_server(*grace, *force).map(Value::OpaqueHandle))
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
                Op::FinishShutdown => result(lifetime.finish(self.task_id).map(Value::Boolean)),
                Op::ShutdownErrors => Value::Array(
                    lifetime
                        .errors()
                        .into_iter()
                        .map(|e| Value::Str(e.into()))
                        .collect::<Vec<_>>()
                        .into(),
                ),
                Op::ObserveSignals => result(
                    if !self
                        .hosted
                        .servers
                        .process_authority
                        .load(Ordering::Acquire)
                    {
                        Err("The embedding host has not authorized process signal ownership".into())
                    } else if !lifetime.ready() {
                        Err("Cannot observe signals after shutdown begins".into())
                    } else {
                        signals::observe(&lifetime).map(Value::Boolean)
                    },
                ),
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
                        result(Err(
                            "Listener already belongs to another server lifetime".into()
                        ))
                    } else {
                        result(lifetime.own_listener(listener).map(Value::Boolean))
                    }
                }
                Op::CreateLifetime => unreachable!(),
            }
        };
        Ok(Some(Some(value)))
    }

    fn create_server(&self, grace: i64, force: bool) -> Result<u64, String> {
        if !(0..=86_400_000).contains(&grace) {
            return Err("GraceMillis must be between 0 and 86400000".into());
        }
        if force
            && !self
                .hosted
                .servers
                .process_authority
                .load(Ordering::Acquire)
        {
            return Err("The embedding host has not authorized process escalation".into());
        }
        if self.debug_tasks && force {
            return Err("Process escalation is unavailable in the source debugger".into());
        }
        let mut entries = self
            .hosted
            .servers
            .entries
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if entries.len() >= 256 {
            return Err("At most 256 server lifetime identities may be created per VM".into());
        }
        let scheduler = self.scheduler_ref().map_err(|e| e.message.clone())?;
        let group = scheduler
            .groups
            .create(self.task_id, || self.hosted.cancellations.create_owned())?;
        let lifetime = Arc::new(Lifetime::new(
            group,
            self.task_id,
            self.hosted.cancellations.create_owned(),
            Duration::from_millis(grace as u64),
            scheduler,
            &self.hosted,
        ));
        if force && let Err(error) = watchdog::start(&lifetime) {
            scheduler.groups.begin_close(group, self.task_id)?;
            scheduler.groups.take_closed(group)?;
            return Err(error);
        }
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        entries.insert(id, lifetime);
        Ok(id)
    }
}

fn result(value: Result<Value, String>) -> Value {
    match value {
        Ok(value) => Value::result_ok(value),
        Err(error) => Value::result_error(Value::Str(error.into())),
    }
}
