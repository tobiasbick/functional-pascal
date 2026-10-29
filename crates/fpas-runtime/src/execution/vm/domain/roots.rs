//! Complete direct execution edges and synchronized idle-session roots.

use crate::{OwnershipError, TraceRoots, Value};
use std::sync::{Arc, RwLock};

impl TraceRoots for super::super::worker::Worker {
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), OwnershipError> {
        if self.registers.len() != self.register_initialized.len() {
            return Err(OwnershipError::Busy);
        }
        for (value, initialized) in self.registers.iter().zip(&self.register_initialized) {
            if *initialized {
                visit(value);
            }
        }
        if let Some(globals) = &self.local_globals {
            for value in globals.iter().flatten() {
                visit(value);
            }
        }
        if let Some(Ok(value)) = &self.pending_completion {
            visit(value);
        }
        if let Some(suspension) = &self.task_suspension {
            suspension.trace_roots(visit)?;
        }
        if let Some(supervision) = &self.supervision {
            supervision.trace_roots(visit)?;
        }
        for continuation in &self.callback_continuations {
            continuation.trace_roots(visit)?;
        }
        if let Some(trace) = &self.debug_local_trace {
            trace.trace_roots(visit)?;
        }
        let callback = self
            .callback_worker
            .try_borrow()
            .map_err(|_| OwnershipError::Busy)?;
        if let Some(callback) = callback.as_ref() {
            callback.trace_roots(visit)?;
        }
        Ok(())
    }
}

impl TraceRoots for super::super::tasks::TaskState {
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), OwnershipError> {
        if self.registers.len() != self.register_initialized.len() {
            return Err(OwnershipError::Busy);
        }
        for (value, initialized) in self.registers.iter().zip(&self.register_initialized) {
            if *initialized {
                visit(value);
            }
        }
        if let Some(Ok(value)) = &self.pending_completion {
            visit(value);
        }
        if let Some(suspension) = &self.suspension {
            suspension.trace_roots(visit)?;
        }
        if let Some(supervision) = &self.supervision {
            supervision.trace_roots(visit)?;
        }
        for continuation in &self.callback_continuations {
            continuation.trace_roots(visit)?;
        }
        Ok(())
    }
}

impl TraceRoots for super::super::tasks::TaskSuspension {
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), OwnershipError> {
        match self {
            Self::Fs(_) => {}   // File jobs retain only native arguments and results.
            Self::Proc(_) => {} // Process waits retain only native data and owned resources.
            Self::Acp(_) => {}  // Native ACP waits retain no FPAS values.
            Self::Mcp(_) => {}  // MCP waits retain only native identities and response rights.
            Self::OwnerCleanup {
                outcome: Ok(value), ..
            }
            | Self::ChannelSend { value, .. }
            | Self::ChannelSendTimeout { value, .. } => visit(value),
            Self::Selection(wait) => wait.trace_roots(visit)?,
            _ => {}
        }
        Ok(())
    }
}

/// This provider retains only globals, never a worker, VM or hosted registry.
pub(in crate::execution::vm) struct GlobalRoots(pub Arc<RwLock<Vec<Option<Value>>>>);

impl TraceRoots for GlobalRoots {
    fn trace_roots(&self, visit: &mut dyn FnMut(&Value)) -> Result<(), OwnershipError> {
        let globals = self.0.try_read().map_err(|error| match error {
            std::sync::TryLockError::WouldBlock => OwnershipError::Busy,
            std::sync::TryLockError::Poisoned(_) => OwnershipError::Poisoned,
        })?;
        for value in globals.iter().flatten() {
            visit(value);
        }
        Ok(())
    }
}
