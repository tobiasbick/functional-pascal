//! Debugger execution gates, stopped access and complete direct-root enumeration.
use super::*;

impl DebugTaskRuntime {
    /// Hold execution exclusion across a bounded group of scheduler rounds. The
    /// tokens stay on the resume thread; newly spawned workers use their own gates.
    pub(in crate::vm::debug) fn begin_execution_batch(
        &self,
    ) -> Result<Vec<(u64, fpas_bytecode::ExecutionAccess)>, (u64, VmError)> {
        let mut accesses = Vec::new();
        for (&task_id, slot) in &self.tasks {
            if slot.worker.execution.is_none() {
                continue;
            }
            match slot.worker.execution_access() {
                Ok(access) => accesses.push((task_id, access)),
                Err(error) => {
                    self.finish_execution_batch(&mut accesses);
                    return Err((task_id, error));
                }
            }
        }
        Ok(accesses)
    }

    /// Publish all retained owners before releasing exclusion, including on a
    /// stop, limit, failure or idle wait. Failed publication keeps maintenance busy.
    pub(in crate::vm::debug) fn finish_execution_batch(
        &self,
        accesses: &mut Vec<(u64, fpas_bytecode::ExecutionAccess)>,
    ) {
        let published: Vec<_> = accesses
            .iter()
            .map(|(id, _)| {
                self.tasks
                    .get(id)
                    .is_some_and(|slot| slot.worker.publish_execution(&[]).is_ok())
            })
            .collect();
        for ((_, access), published) in accesses.iter().zip(published) {
            if published {
                access.safe_point();
            }
        }
        accesses.clear();
    }

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
