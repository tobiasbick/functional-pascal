//! Stopped debugger domain access and complete direct-root enumeration.
use super::*;

impl DebugTaskRuntime {
    pub(in crate::vm::debug) fn configure_collection(
        &mut self,
        schedule: fpas_bytecode::CollectionSchedule,
    ) {
        for slot in self.tasks.values() {
            crate::vm::domain::configure(&slot.worker.domain, schedule);
        }
    }

    pub(in crate::vm::debug) fn set_trace(
        &mut self,
        trace: Option<Arc<std::sync::Mutex<crate::vm::debug::trace::TraceHistory>>>,
    ) {
        let _access = self.access_domains();
        for slot in self.tasks.values_mut() {
            slot.worker.debug_local_trace = trace.as_ref().and_then(|history| {
                let local = history
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .report
                    .watches
                    .iter()
                    .any(|watch| {
                        matches!(
                            watch.identity,
                            crate::vm::debug::trace::DebugTraceIdentity::Local { .. }
                        )
                    });
                local.then(|| {
                    Box::new(crate::vm::debug::trace::LocalTraceState::new(Arc::clone(
                        history,
                    )))
                })
            });
            slot.worker.debug_trace = trace.clone();
            let _ = slot.worker.publish_execution(&[]);
        }
    }

    /// Stopped commands can address peer frames and main-domain globals. Acquire
    /// their gates in stable task order, without retaining runtime/storage borrows.
    pub(in crate::vm::debug) fn access_domains(&self) -> Vec<fpas_bytecode::DomainAccess> {
        let domains: Vec<_> = self
            .tasks
            .values()
            .map(|slot| slot.worker.domain.clone())
            .collect();
        domains.into_iter().map(|domain| domain.access()).collect()
    }
}

impl fpas_bytecode::TraceRoots for DebugTaskRuntime {
    fn trace_roots(
        &self,
        visit: &mut dyn FnMut(&Value),
    ) -> Result<(), fpas_bytecode::OwnershipError> {
        if let Some(value) = &self.root_result {
            visit(value);
        }
        for slot in self.tasks.values() {
            slot.worker.trace_roots(visit)?;
        }
        Ok(())
    }
}
