//! Allocation ownership survives worker migration and releases on the last owner.

use super::{Budget, Exhausted, Ordering};
use std::{cell::RefCell, sync::Arc};

thread_local! {
    static CURRENT: RefCell<Option<Arc<Budget>>> = const { RefCell::new(None) };
}

pub(crate) struct Scope(Option<Arc<Budget>>);
impl Scope {
    pub(crate) fn untracked() -> Self {
        Self(CURRENT.replace(None))
    }

    pub(crate) fn enter(budget: &Arc<Budget>) -> Self {
        Self(CURRENT.replace(Some(Arc::clone(budget))))
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        CURRENT.set(self.0.take());
    }
}

/// An internal allocation refusal unwinds to the VM's fallible execution boundary.
/// `resume_unwind` deliberately bypasses the process panic hook. Unrelated panics
/// retain their original meaning and are never translated into resource errors.
pub(crate) fn catch<T>(run: impl FnOnce() -> T) -> Result<T, Exhausted> {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)) {
        Ok(value) => Ok(value),
        Err(payload) => match payload.downcast::<Exhausted>() {
            Ok(error) => Err(*error),
            Err(payload) => std::panic::resume_unwind(payload),
        },
    }
}

pub(super) fn refuse(error: Exhausted) -> ! {
    std::panic::resume_unwind(Box::new(error))
}

#[derive(Debug, Default)]
pub(crate) struct Reservation {
    budget: Option<Arc<Budget>>,
    bytes: usize,
}
impl Reservation {
    pub(crate) fn shared<T>(backing: usize) -> Self {
        Self::new(shared_size::<T>(backing))
    }

    pub(crate) fn copy_buffer<T>(
        &self,
        values: &[T],
        body: usize,
        copy: impl FnOnce(&[T]) -> Vec<T>,
    ) -> (Self, Vec<T>) {
        let mut memory = Self::new(body.saturating_add(std::mem::size_of_val(values)));
        let values = copy(values);
        memory.resize(
            body.saturating_add(values.capacity().saturating_mul(std::mem::size_of::<T>())),
        );
        (memory, values)
    }

    pub(crate) fn is_tracked(&self) -> bool {
        self.budget.is_some()
    }

    pub(crate) fn new(bytes: usize) -> Self {
        let budget = CURRENT.with(|current| {
            current
                .borrow()
                .as_ref()
                .filter(|b| b.tracks_heap())
                .cloned()
        });
        let mut reservation = Self { budget, bytes: 0 };
        reservation.resize(bytes);
        reservation
    }

    pub(crate) fn resize(&mut self, bytes: usize) {
        if let Some(budget) = &self.budget {
            if bytes > self.bytes {
                if let Err(error) = budget.reserve(bytes - self.bytes) {
                    refuse(error);
                }
            } else {
                budget.heap.fetch_sub(self.bytes - bytes, Ordering::Relaxed);
            }
        }
        self.bytes = bytes;
    }

    pub(crate) fn grow(&mut self, bytes: usize) {
        self.resize(self.bytes.saturating_add(bytes));
    }

    pub(crate) fn reserve_vec<T>(&mut self, values: &mut Vec<T>, additional: usize) {
        let required = values.len().saturating_add(additional);
        if required > values.capacity() {
            let capacity = required.max(values.capacity().saturating_mul(2)).max(4);
            let bytes = (capacity - values.capacity()).saturating_mul(std::mem::size_of::<T>());
            self.grow(bytes);
            values.reserve_exact(capacity - values.len());
        }
    }
}

pub(crate) fn heap_active() -> bool {
    CURRENT.with(|current| {
        current
            .borrow()
            .as_ref()
            .is_some_and(|budget| budget.tracks_heap())
    })
}

pub(crate) fn current_budget() -> Option<Arc<Budget>> {
    CURRENT.with(|current| current.borrow().clone())
}
impl Clone for Reservation {
    fn clone(&self) -> Self {
        Self::new(self.bytes)
    }
}

pub(crate) const fn shared_size<T>(backing: usize) -> usize {
    std::mem::size_of::<T>()
        .saturating_add(2 * std::mem::size_of::<usize>())
        .saturating_add(backing)
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if let Some(budget) = &self.budget {
            budget.heap.fetch_sub(self.bytes, Ordering::Relaxed);
        }
    }
}

/// Check a native producer's known output bound before allocating temporary storage.
pub(crate) fn preflight(bytes: usize) {
    CURRENT.with(|current| {
        if let Some(budget) = current
            .borrow()
            .as_ref()
            .filter(|budget| budget.tracks_heap())
        {
            let limit = budget.effective_heap_limit();
            if budget
                .heap
                .load(Ordering::Relaxed)
                .checked_add(bytes)
                .is_none_or(|n| n > limit)
            {
                refuse(budget.fail(Exhausted::Heap {
                    limit,
                    requested: bytes,
                }));
            }
        }
    });
}

pub(crate) fn output(bytes: usize) {
    CURRENT.with(|current| {
        if let Some(budget) = current.borrow().as_ref()
            && let Err(error) = budget.output(bytes)
        {
            refuse(error);
        }
    });
}
