//! Native callbacks request cancellation without joining work under host-owned locks.

use super::HostedState;

thread_local! { static DEFER_JOINS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }

pub(in crate::execution::vm) fn defer_joins<T>(run: impl FnOnce() -> T) -> T {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            DEFER_JOINS.set(self.0);
        }
    }
    let _restore = Restore(DEFER_JOINS.replace(true));
    run()
}

pub(super) fn retire(state: &HostedState, finish: impl FnOnce() + Send + 'static) {
    if DEFER_JOINS.get() {
        state
            .retired_native_jobs
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .push(Box::new(finish));
    } else {
        finish();
    }
}

pub(super) fn join_retired(state: &HostedState) {
    loop {
        let jobs = std::mem::take(
            &mut *state
                .retired_native_jobs
                .lock()
                .unwrap_or_else(|p| p.into_inner()),
        );
        if jobs.is_empty() {
            break;
        }
        for finish in jobs {
            finish();
        }
    }
}
