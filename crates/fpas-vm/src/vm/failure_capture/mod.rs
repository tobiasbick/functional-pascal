//! Opt-in original-fault capture and terminal notification.

mod capture;
mod identity;
mod model;
mod notification;
mod presentation;
#[cfg(test)]
mod tests;

use std::sync::{Arc, Mutex, Weak, atomic::AtomicU64};

pub use crate::vm::inspection::values::snapshot::{
    Scalar as FailureScalar, ValueSnapshot as FailureValue,
};
pub(in crate::vm) use identity::attach_related;
pub use model::{FailureBinding, FailureFrame, FailureObservation, FailureSnapshot};
pub use presentation::FailureDiagnostic;

use crate::vm::inspection::values::budget::ValueBudget;

const MAX_VALUES: usize = 16_384;
const MAX_BYTES: usize = 8 * 1024 * 1024;
const MAX_FRAMES: usize = 256;
const MAX_STRING: usize = 4096;
const MAX_RELATED: usize = 1024;
// Keep control/model storage and a short terminal diagnostic available after large values.
const CONTROL_RESERVE: usize = 512 * 1024;
const TERMINAL_RESERVE: usize = 128 * 1024;

#[derive(Default)]
struct Usage {
    values: usize,
    bytes: usize,
}

/// VM-wide budget and the host's terminal observation callback; absent unless enabled.
pub(in crate::vm) struct CaptureState {
    next_id: AtomicU64,
    usage: Mutex<Usage>,
    observer: Box<dyn Fn(Arc<FailureObservation>) + Send + Sync>,
    notified: std::sync::atomic::AtomicBool,
}

impl CaptureState {
    /// Enable bounded capture for one VM execution.
    pub(in crate::vm) fn new(
        observer: impl Fn(Arc<FailureObservation>) + Send + Sync + 'static,
    ) -> Arc<Self> {
        Arc::new(Self {
            next_id: AtomicU64::new(1),
            usage: Mutex::default(),
            observer: Box::new(observer),
            notified: std::sync::atomic::AtomicBool::new(false),
        })
    }

    fn budget(&self) -> (std::sync::MutexGuard<'_, Usage>, ValueBudget) {
        self.budget_below(MAX_BYTES - CONTROL_RESERVE)
    }

    fn model_budget(&self) -> (std::sync::MutexGuard<'_, Usage>, ValueBudget) {
        self.budget_below(MAX_BYTES - TERMINAL_RESERVE)
    }

    fn terminal_budget(&self) -> (std::sync::MutexGuard<'_, Usage>, ValueBudget) {
        self.budget_below(MAX_BYTES)
    }

    fn budget_below(&self, limit: usize) -> (std::sync::MutexGuard<'_, Usage>, ValueBudget) {
        let usage = self.usage.lock().unwrap_or_else(|p| p.into_inner());
        let budget = ValueBudget::new(MAX_VALUES - usage.values, limit.saturating_sub(usage.bytes));
        (usage, budget)
    }

    fn reserve_model(self: &Arc<Self>, bytes: usize) -> Option<Lease> {
        let (mut usage, mut budget) = self.model_budget();
        let before = (budget.values, budget.bytes);
        if !budget.charge(bytes) {
            return None;
        }
        let mut lease = Lease::default();
        lease.charge(self, &mut usage, before, &budget);
        Some(lease)
    }

    fn reserve_terminal(self: &Arc<Self>, bytes: usize) -> Lease {
        let (mut usage, mut budget) = self.terminal_budget();
        let before = (budget.values, budget.bytes);
        let mut lease = Lease::default();
        if budget.charge(bytes) {
            lease.charge(self, &mut usage, before, &budget);
        }
        lease
    }

    fn reserve_collection(self: &Arc<Self>, maximum: usize, entry_bytes: usize) -> (usize, Lease) {
        let (mut usage, mut budget) = self.model_budget();
        let before = (budget.values, budget.bytes);
        let count = maximum.min(budget.bytes / entry_bytes);
        budget.charge(count * entry_bytes);
        let mut lease = Lease::default();
        lease.charge(self, &mut usage, before, &budget);
        (count, lease)
    }
}

#[derive(Debug, Default)]
struct Lease {
    owner: Weak<CaptureState>,
    values: usize,
    bytes: usize,
}

impl Lease {
    fn charge(
        &mut self,
        state: &Arc<CaptureState>,
        usage: &mut Usage,
        before: (usize, usize),
        budget: &ValueBudget,
    ) {
        let values = before.0 - budget.values;
        let bytes = before.1 - budget.bytes;
        self.owner = Arc::downgrade(state);
        self.values += values;
        self.bytes += bytes;
        usage.values += values;
        usage.bytes += bytes;
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        if let Some(state) = self.owner.upgrade() {
            let mut usage = state.usage.lock().unwrap_or_else(|p| p.into_inner());
            usage.values -= self.values;
            usage.bytes -= self.bytes;
        }
    }
}
