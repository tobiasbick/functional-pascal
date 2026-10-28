//! Deferred construction of the detached debugger call sandbox.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use crate::{Value, VerifiedExecutable};

use super::CallSandbox;
use crate::execution::vm::debug::evaluation::{DebugCallTarget, DebugEvaluationLimits};
use crate::execution::vm::debug::types::DebugSessionError;
use crate::execution::vm::layouts::RuntimeLayouts;

/// Creates detached call state only when an evaluated expression invokes a call.
pub(in crate::execution::vm::debug) struct LazyCallSandbox<'a> {
    inspection: &'a crate::execution::vm::debug::inspection::InspectionSnapshot,
    frame_id: Option<u64>,
    executable: Arc<VerifiedExecutable>,
    layouts: Arc<RuntimeLayouts>,
    globals: Arc<RwLock<Vec<Option<Value>>>>,
    global_domain: Option<crate::MemoryDomain>,
    limits: DebugEvaluationLimits,
    cancelled: Arc<AtomicBool>,
    sandbox: Option<CallSandbox>,
}

impl<'a> LazyCallSandbox<'a> {
    /// Retains the inputs needed to create a detached call sandbox on first invocation.
    #[expect(
        clippy::too_many_arguments,
        reason = "source storage carries explicit ownership authority separately from the selected task"
    )]
    pub(in crate::execution::vm::debug) fn new(
        inspection: &'a crate::execution::vm::debug::inspection::InspectionSnapshot,
        frame_id: Option<u64>,
        executable: Arc<VerifiedExecutable>,
        layouts: Arc<RuntimeLayouts>,
        globals: Arc<RwLock<Vec<Option<Value>>>>,
        global_domain: Option<crate::MemoryDomain>,
        limits: DebugEvaluationLimits,
        cancelled: Arc<AtomicBool>,
    ) -> Self {
        Self {
            inspection,
            frame_id,
            executable,
            layouts,
            globals,
            global_domain,
            limits,
            cancelled,
            sandbox: None,
        }
    }

    /// Retain only reachable sandbox cells, not the globals or temporary construction map.
    pub(in crate::execution::vm::debug) fn result_roots(
        &self,
        values: &[Value],
    ) -> Result<Vec<crate::RootValues>, DebugSessionError> {
        let mut roots = self.inspection.root_guards();
        if let Some(sandbox) = &self.sandbox {
            let domain = &sandbox.detacher.domain;
            let mut retained = Vec::new();
            crate::inspect_values(values, |value| match value {
                Value::Cell(cell) if cell.owner() == domain.id() => retained.push(value.clone()),
                Value::Function(function)
                    if function.owner_domain.as_ref() == Some(domain.id()) =>
                {
                    retained.push(value.clone())
                }
                _ => {}
            })
            .map_err(|_| {
                super::detach::error(
                    crate::execution::vm::debug::types::DebugErrorKind::UnavailableValue,
                    "debug result graph is unavailable",
                    "Retry at a stable stop.",
                )
            })?;
            roots.push(domain.root_values(crate::RootKind::Evaluation, retained));
        }
        Ok(roots)
    }

    /// Invokes a debugger call through one lazily initialized detached sandbox.
    pub(in crate::execution::vm::debug) fn invoke(
        &mut self,
        target: DebugCallTarget,
        arguments: Vec<Value>,
    ) -> Result<Value, DebugSessionError> {
        if self.sandbox.is_none() {
            let image = self.executable.executable();
            let function = self
                .inspection
                .frame_function(self.frame_id)
                .unwrap_or(image.entry);
            let source = image
                .functions
                .get(function.get() as usize)
                .and_then(|function| {
                    function
                        .debug
                        .sequence_points
                        .first()
                        .map(|point| point.location.source)
                        .or_else(|| {
                            image
                                .source_map
                                .lookup(crate::InstructionAddress::new(
                                    function.code.end.get().saturating_sub(1),
                                ))
                                .map(|run| run.source)
                        })
                })
                .unwrap_or(crate::SourceId::new(0));
            let domains = std::iter::once(self.inspection.domain.clone())
                .chain(self.global_domain.clone())
                .collect::<Vec<_>>();
            self.sandbox = Some(CallSandbox::new(
                Arc::clone(&self.executable),
                Arc::clone(&self.layouts),
                source,
                &self.globals,
                &domains,
                self.limits,
                Arc::clone(&self.cancelled),
            )?);
        }
        let sandbox = self.sandbox.as_mut().ok_or_else(|| {
            super::detach::error(
                crate::execution::vm::debug::types::DebugErrorKind::UnavailableValue,
                "debug call sandbox is unavailable",
                "Retry at a stable stop.",
            )
        })?;
        if let DebugCallTarget::Reference(assignment) = target {
            let result =
                sandbox.reference(self.inspection, self.frame_id, &assignment, &arguments)?;
            sandbox.temporary_roots.push(result.clone());
            return Ok(result);
        }
        let result = sandbox.invoke(target, arguments)?;
        sandbox.temporary_roots.push(result.clone());
        Ok(result)
    }
}
