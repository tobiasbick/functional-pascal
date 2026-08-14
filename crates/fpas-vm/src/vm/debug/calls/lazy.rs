//! Deferred construction of the detached debugger call sandbox.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, RwLock};

use fpas_bytecode::{Value, VerifiedExecutable};

use super::CallSandbox;
use crate::vm::debug::evaluation::{DebugCallTarget, DebugEvaluationLimits};
use crate::vm::debug::types::DebugSessionError;
use crate::vm::layouts::RuntimeLayouts;

/// Creates detached call state only when an evaluated expression invokes a call.
pub(in crate::vm::debug) struct LazyCallSandbox {
    executable: Arc<VerifiedExecutable>,
    layouts: Arc<RuntimeLayouts>,
    globals: Arc<RwLock<Vec<Option<Value>>>>,
    limits: DebugEvaluationLimits,
    cancelled: Arc<AtomicBool>,
    sandbox: Option<CallSandbox>,
}

impl LazyCallSandbox {
    /// Retains the inputs needed to create a detached call sandbox on first invocation.
    pub(in crate::vm::debug) fn new(
        executable: Arc<VerifiedExecutable>,
        layouts: Arc<RuntimeLayouts>,
        globals: Arc<RwLock<Vec<Option<Value>>>>,
        limits: DebugEvaluationLimits,
        cancelled: Arc<AtomicBool>,
    ) -> Self {
        Self {
            executable,
            layouts,
            globals,
            limits,
            cancelled,
            sandbox: None,
        }
    }

    /// Invokes a debugger call through one lazily initialized detached sandbox.
    pub(in crate::vm::debug) fn invoke(
        &mut self,
        target: DebugCallTarget,
        arguments: Vec<Value>,
    ) -> Result<Value, DebugSessionError> {
        if let Some(sandbox) = self.sandbox.as_mut() {
            return sandbox.invoke(target, arguments);
        }
        let mut sandbox = CallSandbox::new(
            Arc::clone(&self.executable),
            Arc::clone(&self.layouts),
            &self.globals,
            self.limits,
            Arc::clone(&self.cancelled),
        )?;
        let result = sandbox.invoke(target, arguments);
        self.sandbox = Some(sandbox);
        result
    }
}
