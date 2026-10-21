//! Operation-scoped roots for evaluation handoffs, including failure paths.

use fpas_bytecode::{MemoryDomain, RootValues};
use std::sync::{Arc, Mutex};

#[derive(Default)]
pub(super) struct EvaluationRoots {
    state: Mutex<State>,
}
#[derive(Default)]
struct State {
    depth: usize,
    roots: Vec<RootValues>,
}
pub(super) struct EvaluationScope(Arc<EvaluationRoots>);

impl EvaluationRoots {
    pub(super) fn enter(self: &Arc<Self>) -> EvaluationScope {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.depth += 1;
        EvaluationScope(Arc::clone(self))
    }
    pub(super) fn retain(&self, roots: impl IntoIterator<Item = RootValues>) {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .roots
            .extend(roots);
    }
    pub(super) fn domains(&self) -> Vec<MemoryDomain> {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .roots
            .iter()
            .map(|root| root.domain().clone())
            .collect()
    }
}
impl Drop for EvaluationScope {
    fn drop(&mut self) {
        let roots = {
            let mut state = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
            state.depth -= 1;
            if state.depth == 0 {
                std::mem::take(&mut state.roots)
            } else {
                Vec::new()
            }
        };
        drop(roots);
    }
}

#[cfg(test)]
mod tests;
