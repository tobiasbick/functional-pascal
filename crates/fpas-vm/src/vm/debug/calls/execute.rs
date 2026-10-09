//! Effect-checked detached worker invocation and aggregate construction.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use fpas_bytecode::{
    DebugEffectSet, Intrinsic, Value, VerifiedExecutable, analyze_debug_effects,
    intrinsic_debug_effects,
};

use super::detach::{ValueDetacher, error};
use super::enum_constructor;
use super::resolution::{NamedTarget, resolve_named};
use crate::vm::debug::evaluation::{DebugCallTarget, DebugEvaluationLimits};
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};
use crate::vm::hosted::HostedState;
use crate::vm::layouts::RuntimeLayouts;
use crate::vm::worker::Worker;

/// Bounded, effect-checked detached call state. See `docs/pascal/tools/debugger.md`.
pub(in crate::vm::debug) struct CallSandbox {
    pub(super) executable: Arc<VerifiedExecutable>,
    pub(super) layouts: Arc<RuntimeLayouts>,
    pub(super) source: fpas_bytecode::SourceId,
    pub(super) globals: Arc<RwLock<Vec<Option<Value>>>>,
    pub(super) effects: Vec<DebugEffectSet>,
    pub(super) detacher: ValueDetacher,
    pub(super) limits: DebugEvaluationLimits,
    started: Instant,
    calls: usize,
    pub(super) instructions: u64,
    pub(super) reference_types: HashMap<usize, fpas_bytecode::DebugTypeId>,
    pub(super) local_cells: HashMap<usize, Arc<std::sync::Mutex<Value>>>,
    cancelled: Arc<AtomicBool>,
}

impl CallSandbox {
    /// Snapshots globals without sharing mutable storage with the live program.
    /// See `docs/pascal/tools/debugger.md`.
    pub(in crate::vm::debug) fn new(
        executable: Arc<VerifiedExecutable>,
        layouts: Arc<RuntimeLayouts>,
        source: fpas_bytecode::SourceId,
        source_globals: &Arc<RwLock<Vec<Option<Value>>>>,
        limits: DebugEvaluationLimits,
        cancelled: Arc<AtomicBool>,
    ) -> Result<Self, DebugSessionError> {
        let effects = analyze_debug_effects(&executable)
            .into_iter()
            .map(|summary| summary.transitive)
            .collect();
        let mut detacher = ValueDetacher::new(limits.max_detached_values);
        let globals = source_globals
            .read()
            .map_err(|_| {
                error(
                    DebugErrorKind::UnavailableValue,
                    "debug call cannot snapshot poisoned global storage",
                    "Restart the debug session before evaluating calls.",
                )
            })?
            .iter()
            .map(|value| {
                value
                    .as_ref()
                    .map(|value| detacher.detach(value))
                    .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            executable,
            layouts,
            source,
            globals: Arc::new(RwLock::new(globals)),
            effects,
            detacher,
            limits,
            started: Instant::now(),
            calls: 0,
            instructions: 0,
            reference_types: HashMap::new(),
            local_cells: HashMap::new(),
            cancelled,
        })
    }

    /// Invokes one resolved callable or typed constructor under shared call limits.
    /// See `docs/pascal/tools/debugger.md`.
    pub(in crate::vm::debug) fn invoke(
        &mut self,
        mut target: DebugCallTarget,
        mut arguments: Vec<Value>,
    ) -> Result<Value, DebugSessionError> {
        self.check_boundary()?;
        if let DebugCallTarget::NamedArguments {
            target: inner,
            names,
        } = target
        {
            if let DebugCallTarget::Named(name) = inner.as_ref()
                && let Some(record) = self.record_type(name)?
            {
                return self.construct_record(record, name, &names, arguments);
            }
            arguments = self.order_named_arguments(&inner, &names, arguments)?;
            target = *inner;
        }
        match target {
            DebugCallTarget::Reference(_) => Err(error(
                DebugErrorKind::EvaluationType,
                "unresolved debugger reference",
                "Resolve the designator in a stopped frame before invoking a call.",
            )),
            DebugCallTarget::NamedArguments { .. } => Err(error(
                DebugErrorKind::EvaluationType,
                "debug call contains nested argument-name metadata",
                "Supply argument names only on the call itself.",
            )),
            DebugCallTarget::Named(name) => self.invoke_named(&name, arguments),
            DebugCallTarget::Value(Value::Function(function)) => self.invoke_function(
                function.function,
                function.bound_receiver.as_ref(),
                &function.captures,
                arguments,
                &function.name,
            ),
            DebugCallTarget::Value(other) => Err(error(
                DebugErrorKind::EvaluationType,
                format!(
                    "debug call target has type {}, not function",
                    other.type_name()
                ),
                "Call a named function, procedure, method, or visible function value.",
            )),
            DebugCallTarget::Method { receiver, name } => {
                self.invoke_member(receiver, &name, arguments)
            }
            DebugCallTarget::Record { name, fields } => {
                let record = self.record_type(&name)?.ok_or_else(|| {
                    error(
                        DebugErrorKind::UnknownCallable,
                        format!("debug record type `{name}` is not visible in this source"),
                        "Use a visible record type name.",
                    )
                })?;
                self.construct_record(record, &name, &fields, arguments)
            }
        }
    }

    fn check_boundary(&mut self) -> Result<(), DebugSessionError> {
        self.calls = self.calls.saturating_add(1);
        if self.calls > self.limits.max_calls {
            return Err(error(
                DebugErrorKind::CallLimit,
                format!("debug call count exceeds limit {}", self.limits.max_calls),
                "Use fewer calls in one watch expression.",
            ));
        }
        self.check_running()
    }

    /// Enforces cancellation and the shared deadline. See `docs/pascal/tools/debugger.md`.
    pub(super) fn check_running(&self) -> Result<(), DebugSessionError> {
        if self.cancelled.load(Ordering::Acquire) {
            return Err(error(
                DebugErrorKind::CallCancelled,
                "debug call evaluation was cancelled",
                "Evaluate again after the debugger is stopped.",
            ));
        }
        if self.started.elapsed() > self.limits.call_timeout {
            return Err(error(
                DebugErrorKind::CallTimeout,
                format!(
                    "debug call evaluation exceeded {} ms",
                    self.limits.call_timeout.as_millis()
                ),
                "Use a faster bounded callable or increase the debugger call timeout.",
            ));
        }
        Ok(())
    }

    fn invoke_named(
        &mut self,
        name: &str,
        arguments: Vec<Value>,
    ) -> Result<Value, DebugSessionError> {
        if let Some(record) = self.record_type(name)? {
            return self.construct_record(record, name, &[], arguments);
        }
        match resolve_named(&self.executable, &self.layouts, name)? {
            NamedTarget::Function(function) => {
                self.invoke_function(function, None, &[], arguments, name)
            }
            NamedTarget::EnumConstructor(layout) => enum_constructor::construct(
                &self.executable,
                layout,
                arguments,
                &mut self.detacher,
                self.limits.max_depth,
            ),
            NamedTarget::Intrinsic(intrinsic) => self.invoke_intrinsic(name, intrinsic, arguments),
        }
    }

    fn invoke_intrinsic(
        &mut self,
        name: &str,
        intrinsic: Intrinsic,
        arguments: Vec<Value>,
    ) -> Result<Value, DebugSessionError> {
        let effects = intrinsic_debug_effects(intrinsic);
        self.require_safe(name, effects)?;
        let arguments = self.detach_values(&arguments)?;
        let mut worker = Worker::for_function_with_state(
            Arc::clone(&self.executable),
            self.executable.executable().entry,
            Vec::new(),
            Arc::clone(&self.globals),
            Arc::clone(&self.layouts),
            Arc::new(HostedState::new(fpas_std::Console::new(), Vec::new())),
        )
        .map_err(|diagnostic| runtime_error(*diagnostic))?;
        worker
            .execute_debug_intrinsic(intrinsic, &arguments)
            .map_err(|diagnostic| runtime_error(*diagnostic))
    }

    fn invoke_member(
        &mut self,
        receiver: Value,
        member: &str,
        mut arguments: Vec<Value>,
    ) -> Result<Value, DebugSessionError> {
        let name = self.member_name(&receiver, member)?;
        arguments.insert(0, receiver);
        self.invoke_named(&name, arguments)
    }

    /// Resolves a record method through its canonical executable routine mapping.
    /// See `docs/pascal/language/types/records.md`.
    pub(super) fn member_name(
        &self,
        receiver: &Value,
        member: &str,
    ) -> Result<String, DebugSessionError> {
        let Value::Record(record) = receiver else {
            return Err(error(
                DebugErrorKind::EvaluationType,
                format!(
                    "debug member call requires record receiver, got {}",
                    receiver.type_name()
                ),
                "Call instance members on record values.",
            ));
        };
        let image = self.executable.executable();
        image
            .records
            .get(record.body().layout.record.get() as usize)
            .and_then(|layout| {
                layout.methods.iter().find(|method| {
                    image
                        .strings
                        .get(method.name)
                        .is_some_and(|name| name.eq_ignore_ascii_case(member))
                })
            })
            .and_then(|method| image.strings.get(method.routine))
            .map(str::to_owned)
            .ok_or_else(|| {
                error(
                    DebugErrorKind::UnknownCallable,
                    format!(
                        "debug record `{}` has no method `{member}`",
                        record.body().layout.type_name
                    ),
                    "Call a declared instance method on this record type.",
                )
            })
    }

    /// Rejects effects that may escape detached evaluation. See `docs/pascal/tools/debugger.md`.
    pub(super) fn require_safe(
        &self,
        name: &str,
        effects: DebugEffectSet,
    ) -> Result<(), DebugSessionError> {
        if effects.is_debug_safe() {
            return Ok(());
        }
        Err(error(
            DebugErrorKind::ForbiddenCallEffect,
            format!("debug callable `{name}` has effects forbidden in detached evaluation"),
            "Use a deterministic callable without host I/O, time, randomness, tasks, blocking, or unknown dynamic calls.",
        ))
    }
}

/// Converts detached worker diagnostics to debugger call errors.
/// See `docs/pascal/tools/debugger.md`.
pub(super) fn runtime_error(diagnostic: fpas_diagnostics::Diagnostic) -> DebugSessionError {
    let fpas_diagnostics::Diagnostic { message, help, .. } = diagnostic;
    error(
        DebugErrorKind::CallRuntime,
        message,
        help.unwrap_or_else(|| "Inspect the callable inputs and retry.".to_string()),
    )
}
