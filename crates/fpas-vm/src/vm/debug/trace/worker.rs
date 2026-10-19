//! Successful-store observation at the existing global write owners.

use super::TraceHistory;
use crate::vm::worker::Worker;
use fpas_bytecode::Value;
use std::sync::{Arc, Mutex};

pub(in crate::vm) struct TraceWrite {
    global: usize,
    before: Result<Vec<(usize, Option<Value>)>, &'static str>,
    history: Arc<Mutex<TraceHistory>>,
}

impl Worker {
    /// Retain scalar leaves only, before a possibly in-place aggregate commit.
    pub(in crate::vm) fn begin_trace_store(&self, index: usize) -> Option<TraceWrite> {
        let history = self.debug_trace.as_ref()?;
        let state = history.lock().unwrap_or_else(|p| p.into_inner());
        if state.limit.is_some()
            || !state.report.watches.iter().any(|watch| {
                watch
                    .identity
                    .global()
                    .is_some_and(|(slot, _)| slot == index)
            })
        {
            return None;
        }
        let globals = self.global_slots();
        let before = state.leaves(index, globals.get(index).and_then(Option::as_ref));
        Some(TraceWrite {
            global: index,
            before,
            history: Arc::clone(history),
        })
    }

    /// Failed stores discard their before-values and never invoke this method.
    pub(in crate::vm) fn finish_trace_store(&self, write: Option<TraceWrite>) {
        let Some(write) = write else { return };
        let mut state = write.history.lock().unwrap_or_else(|p| p.into_inner());
        let before = match write.before {
            Ok(before) => before,
            Err(reason) => {
                state.fail(reason, false);
                return;
            }
        };
        let after = {
            let globals = self.global_slots();
            state.leaves(
                write.global,
                globals.get(write.global).and_then(Option::as_ref),
            )
        };
        let after = match after {
            Ok(after) => after,
            Err(reason) => {
                state.fail(reason, false);
                return;
            }
        };
        let image = self.executable.executable();
        let function = image
            .functions
            .get(self.function.get() as usize)
            .and_then(|info| image.strings.get(info.name));
        let position = crate::vm::inspection::location::position_at(
            image,
            self.function,
            self.current_address,
        );
        let location = position.and_then(|p| {
            crate::vm::inspection::location::source_path(image, p)
                .map(|source| (source, p.line, p.column))
        });
        state.commit(
            image,
            before,
            after,
            super::pending::TraceSite {
                task: self.task_id,
                function,
                location,
                commit: self.debug_local_trace.as_ref().map(|state| state.commit),
            },
            None,
        );
    }

    pub(in crate::vm) fn trace_interrupted(&self) -> bool {
        self.debug_trace.as_ref().is_some_and(|history| {
            history
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .limit
                .is_some()
        })
    }
}
