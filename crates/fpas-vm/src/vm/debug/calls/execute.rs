//! Effect-checked detached worker invocation and aggregate construction.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use fpas_bytecode::{
    DebugEffectSet, FunctionId, Intrinsic, SharedRecord, Value, VerifiedExecutable,
    analyze_debug_effects, intrinsic_debug_effects,
};

use super::detach::{ValueDetacher, error};
use super::enum_constructor;
use super::resolution::{NamedTarget, resolve_named};
use crate::vm::debug::evaluation::{DebugCallTarget, DebugEvaluationLimits};
use crate::vm::debug::types::{DebugErrorKind, DebugSessionError};
use crate::vm::dispatch::DispatchStep;
use crate::vm::hosted::HostedState;
use crate::vm::layouts::RuntimeLayouts;
use crate::vm::worker::Worker;

pub(in crate::vm::debug) struct CallSandbox {
    executable: Arc<VerifiedExecutable>,
    layouts: Arc<RuntimeLayouts>,
    globals: Arc<RwLock<Vec<Option<Value>>>>,
    effects: Vec<DebugEffectSet>,
    detacher: ValueDetacher,
    limits: DebugEvaluationLimits,
    started: Instant,
    calls: usize,
    instructions: u64,
    cancelled: Arc<AtomicBool>,
}

impl CallSandbox {
    pub(in crate::vm::debug) fn new(
        executable: Arc<VerifiedExecutable>,
        layouts: Arc<RuntimeLayouts>,
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
            globals: Arc::new(RwLock::new(globals)),
            effects,
            detacher,
            limits,
            started: Instant::now(),
            calls: 0,
            instructions: 0,
            cancelled,
        })
    }

    pub(in crate::vm::debug) fn invoke(
        &mut self,
        target: DebugCallTarget,
        arguments: Vec<Value>,
    ) -> Result<Value, DebugSessionError> {
        self.check_boundary()?;
        match target {
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
                self.invoke_member(receiver, &name, arguments, false)
            }
            DebugCallTarget::Property { receiver, name } => {
                self.invoke_member(receiver, &name, arguments, true)
            }
            DebugCallTarget::Record { fields } => self.construct_record(&fields, arguments),
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

    fn check_running(&self) -> Result<(), DebugSessionError> {
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
        .map_err(runtime_error)?;
        worker
            .execute_debug_intrinsic(intrinsic, &arguments)
            .map_err(runtime_error)
    }

    fn invoke_member(
        &mut self,
        receiver: Value,
        member: &str,
        mut arguments: Vec<Value>,
        property: bool,
    ) -> Result<Value, DebugSessionError> {
        let Value::Record(record) = &receiver else {
            return Err(error(
                DebugErrorKind::EvaluationType,
                format!(
                    "debug member call requires record receiver, got {}",
                    receiver.type_name()
                ),
                "Call instance members on record values.",
            ));
        };
        let name = if property {
            let getter = self
                .executable
                .executable()
                .records
                .get(usize::from(record.body().layout.record.get()))
                .and_then(|layout| {
                    layout.properties.iter().find(|property| {
                        self.executable
                            .executable()
                            .strings
                            .get(property.name)
                            .is_some_and(|name| name.eq_ignore_ascii_case(member))
                    })
                })
                .and_then(|property| self.executable.executable().strings.get(property.getter))
                .ok_or_else(|| {
                    error(
                        DebugErrorKind::UnknownCallable,
                        format!(
                            "record `{}` has no readable property `{member}`",
                            record.body().layout.type_name
                        ),
                        "Use a stored field or a readable property from the executable metadata.",
                    )
                })?;
            getter.to_string()
        } else {
            format!("{}.{}", record.body().layout.type_name, member)
        };
        arguments.insert(0, receiver);
        self.invoke_named(&name, arguments)
    }

    fn invoke_function(
        &mut self,
        function: FunctionId,
        bound_receiver: Option<&Value>,
        captures: &[Value],
        mut arguments: Vec<Value>,
        display_name: &str,
    ) -> Result<Value, DebugSessionError> {
        let info = self
            .executable
            .executable()
            .functions
            .get(usize::from(function.get()))
            .ok_or_else(|| {
                error(
                    DebugErrorKind::UnknownCallable,
                    format!("debug callable `{display_name}` references a missing function"),
                    "Rebuild the executable with the current compiler.",
                )
            })?;
        let visible_arity = usize::from(info.arity)
            .checked_sub(usize::from(bound_receiver.is_some()))
            .ok_or_else(|| {
                error(
                    DebugErrorKind::CallArity,
                    format!("debug callable `{display_name}` has no receiver parameter"),
                    "Rebuild the executable with current bound-method metadata.",
                )
            })?;
        if visible_arity != arguments.len() {
            return Err(error(
                DebugErrorKind::CallArity,
                format!(
                    "debug callable `{display_name}` expects {} arguments, received {}",
                    visible_arity,
                    arguments.len()
                ),
                "Pass the exact declared argument count.",
            ));
        }
        if usize::from(info.capture_count) != captures.len() {
            return Err(error(
                DebugErrorKind::CallArity,
                format!(
                    "debug callable `{display_name}` expects {} captures, received {}",
                    info.capture_count,
                    captures.len()
                ),
                "Invoke nested routines through their visible first-class function value.",
            ));
        }
        let effects = self
            .effects
            .get(usize::from(function.get()))
            .copied()
            .unwrap_or(DebugEffectSet::UNKNOWN);
        self.require_safe(display_name, effects)?;
        if let Some(receiver) = bound_receiver {
            arguments.insert(0, receiver.clone());
        }
        let arguments = self.detach_values(&arguments)?;
        let captures = self.detach_values(captures)?;
        let mut worker = Worker::for_function_with_captures(
            Arc::clone(&self.executable),
            function,
            &arguments,
            &captures,
            Arc::clone(&self.globals),
            Arc::clone(&self.layouts),
            Arc::new(HostedState::new(fpas_std::Console::new(), Vec::new())),
        )
        .map_err(runtime_error)?;
        loop {
            self.check_running()?;
            if self.instructions >= self.limits.max_call_instructions {
                return Err(error(
                    DebugErrorKind::CallLimit,
                    format!(
                        "debug call instruction count exceeds limit {}",
                        self.limits.max_call_instructions
                    ),
                    "Use a smaller bounded callable.",
                ));
            }
            match worker.dispatch_one().map_err(runtime_error)? {
                DispatchStep::Continue => {}
                DispatchStep::Return(value) => return Ok(value),
                DispatchStep::Suspend => {
                    return Err(error(
                        DebugErrorKind::ForbiddenCallEffect,
                        "debug call attempted to suspend on task scheduling",
                        "Remove task and scheduler operations from debugger-call targets.",
                    ));
                }
            }
            self.instructions = self.instructions.saturating_add(1);
            if worker.call_stack.len() > self.limits.max_call_depth {
                return Err(error(
                    DebugErrorKind::CallLimit,
                    format!(
                        "debug call depth exceeds limit {}",
                        self.limits.max_call_depth
                    ),
                    "Use a shallower call chain or recursion depth.",
                ));
            }
        }
    }

    fn detach_values(&mut self, values: &[Value]) -> Result<Vec<Value>, DebugSessionError> {
        values
            .iter()
            .map(|value| self.detacher.detach(value))
            .collect()
    }

    fn require_safe(&self, name: &str, effects: DebugEffectSet) -> Result<(), DebugSessionError> {
        if effects.is_debug_safe() {
            return Ok(());
        }
        Err(error(
            DebugErrorKind::ForbiddenCallEffect,
            format!("debug callable `{name}` has effects forbidden in detached evaluation"),
            "Use a deterministic callable without host I/O, time, randomness, tasks, blocking, or unknown dynamic calls.",
        ))
    }

    fn construct_record(
        &self,
        fields: &[String],
        values: Vec<Value>,
    ) -> Result<Value, DebugSessionError> {
        let mut candidates = self.layouts.records.iter().filter(|layout| {
            layout.fields.len() == fields.len()
                && layout.fields.iter().all(|candidate| {
                    fields
                        .iter()
                        .any(|field| field.eq_ignore_ascii_case(candidate))
                })
        });
        let Some(layout) = candidates.next() else {
            return Err(error(
                DebugErrorKind::UnknownCallable,
                "debug record literal does not match an executable record layout",
                "Use the complete exact stored-field set of one visible record type.",
            ));
        };
        if candidates.next().is_some() {
            return Err(error(
                DebugErrorKind::AmbiguousCallable,
                "debug record literal matches multiple executable record layouts",
                "Pass an existing typed record value or use a uniquely shaped record literal.",
            ));
        }
        let ordered = layout
            .fields
            .iter()
            .map(|candidate| {
                fields
                    .iter()
                    .position(|field| field.eq_ignore_ascii_case(candidate))
                    .and_then(|index| values.get(index).cloned())
                    .ok_or_else(|| {
                        error(
                            DebugErrorKind::CallRuntime,
                            "debug record literal field ordering failed",
                            "Re-enter the record literal with each field exactly once.",
                        )
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Value::Record(SharedRecord::new(
            Arc::clone(layout),
            ordered,
        )))
    }
}

fn runtime_error(diagnostic: fpas_diagnostics::Diagnostic) -> DebugSessionError {
    error(
        DebugErrorKind::CallRuntime,
        diagnostic.message,
        diagnostic
            .help
            .unwrap_or_else(|| "Inspect the callable inputs and retry.".to_string()),
    )
}
