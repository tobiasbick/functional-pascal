//! Read-only task names and borrowed saved register windows.

use super::super::CaptureState;
use super::*;
use std::time::Instant;

impl CaptureState {
    /// Copy a bounded name from portable metadata.
    pub(super) fn task_name(
        self: &Arc<Self>,
        id: u64,
        function: Option<fpas_bytecode::FunctionId>,
        important: bool,
    ) -> (Option<String>, Option<&'static str>, super::super::Lease) {
        let template = self.catalog.template.try_lock().ok();
        let image = template
            .as_ref()
            .and_then(|template| template.as_ref())
            .map(|template| template.executable.executable());
        let name = if id == 0 {
            Some("main")
        } else {
            image
                .and_then(|image| {
                    function
                        .and_then(|function| image.functions.get(usize::from(function.get())))
                        .and_then(|function| image.strings.get(function.name))
                })
                .map(super::super::super::inspection::location::routine_name)
        };
        let Some(name) = name else {
            return (
                None,
                Some("missing_metadata"),
                super::super::Lease::default(),
            );
        };
        let (mut usage, mut budget) = if important {
            self.terminal_budget()
        } else {
            self.model_budget()
        };
        let before = (budget.values, budget.bytes);
        let (name, truncated) = budget.text(name, super::super::MAX_STRING);
        let mut lease = super::super::Lease::default();
        lease.charge(self, &mut usage, before, &budget);
        (
            Some(name),
            truncated.then_some(budget.exhausted.unwrap_or("string_limit")),
            lease,
        )
    }

    /// Borrow the original saved state and restore it even if inspection unwinds.
    pub(super) fn inspect_saved(
        self: &Arc<Self>,
        record: &TaskRecord,
        deadline: Instant,
    ) -> Option<Arc<FailureSnapshot>> {
        if Instant::now() >= deadline {
            return None;
        }
        let mut saved = record.saved.try_lock().ok()?;
        let task = saved.take()?;
        let template = self
            .catalog
            .template
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let Some(template) = template.as_ref() else {
            *saved = Some(task);
            return None;
        };
        let owner = SavedInspection {
            slot: &mut saved,
            worker: template.worker_for_task(task),
        };
        Some(self.capture_peer(&owner.worker))
    }
}

struct SavedInspection<'a> {
    slot: &'a mut Option<TaskState>,
    worker: Worker,
}
impl Drop for SavedInspection<'_> {
    fn drop(&mut self) {
        *self.slot = Some(self.worker.take_task_state());
    }
}
