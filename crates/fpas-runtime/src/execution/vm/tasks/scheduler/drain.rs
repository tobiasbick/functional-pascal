//! Graceful cancellation keeps workers and timers alive until resource release finishes.

use std::sync::atomic::Ordering;

use super::{TaskScheduler, TaskState};
use crate::execution::vm::tasks::TaskSuspension;

pub(super) fn needs_execution(task: &TaskState) -> bool {
    !task.cleanup.scopes.is_empty()
        || !task.cleanup.exits.is_empty()
        || !task.cleanup.initializers.is_empty()
        || task.cleanup.pending_failure.is_some()
        || task.cleanup.deferred_cancel.is_some()
        || task.pending_completion.is_some()
        || !task.task_scopes.is_empty()
        || matches!(
            task.suspension,
            Some(
                TaskSuspension::Entropy(_)
                    | TaskSuspension::ConsoleIo(_)
                    | TaskSuspension::Net(_)
                    | TaskSuspension::Fs(_)
                    | TaskSuspension::Proc(_)
                    | TaskSuspension::Acp(_)
                    | TaskSuspension::Mcp(_)
                    | TaskSuspension::NativeRetirement { .. }
            )
        )
}

pub(in crate::execution::vm) struct WorkerTicket<'a> {
    scheduler: &'a TaskScheduler,
    task: u64,
}

impl Drop for WorkerTicket<'_> {
    fn drop(&mut self) {
        self.scheduler.worker_finished(self.task);
    }
}

impl TaskScheduler {
    pub(in crate::execution::vm) fn worker_ticket(&self, task: u64) -> WorkerTicket<'_> {
        WorkerTicket {
            scheduler: self,
            task,
        }
    }

    pub(in crate::execution::vm) fn worker_started(&self, task: u64) {
        *self
            .active_workers
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .entry(task)
            .or_default() += 1;
    }

    pub(in crate::execution::vm) fn worker_finished(&self, task: u64) {
        let mut workers = self
            .active_workers
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if let Some(count) = workers.get_mut(&task) {
            *count -= 1;
            if *count == 0 {
                workers.remove(&task);
            }
        }
        drop(workers);
        self.try_finish_drain();
    }

    pub(super) fn enqueue_draining(&self, task: TaskState) {
        let mut queue = self.lock_queue();
        if self.is_shutdown() {
            drop(queue);
            self.cancel(task);
            return;
        }
        queue.push_back(super::ScheduledTask::Direct(task));
        self.queued.store(queue.len(), Ordering::Release);
        drop(queue);
        self.available.notify_all();
        self.notify_result_change();
    }

    pub(super) fn begin_drain(&self) {
        self.preparing_drain.store(true, Ordering::Release);
        self.abort.store(true, Ordering::Release);
        if !self.draining.swap(true, Ordering::AcqRel) {
            self.groups.shutdown();
            self.scopes.shutdown();
            let sleeping = self.timers.cancel_all();
            let count = sleeping.len();
            for task in sleeping {
                let task = task.take();
                if needs_execution(&task) {
                    self.enqueue_draining(task);
                } else {
                    self.cancel(task);
                }
            }
            self.sleeping_tasks.fetch_sub(count, Ordering::AcqRel);
            let queued = {
                let mut queue = self.lock_queue();
                self.queued.store(0, Ordering::Release);
                std::mem::take(&mut *queue)
            };
            for task in queued {
                let task = task.take();
                if needs_execution(&task) {
                    self.enqueue_draining(task);
                } else {
                    self.cancel(task);
                }
            }
            self.available.notify_all();
        }
        self.preparing_drain.store(false, Ordering::Release);
        self.try_finish_drain();
    }

    pub(super) fn try_finish_drain(&self) {
        if !self.draining.load(Ordering::Acquire)
            || self.preparing_drain.load(Ordering::Acquire)
            || self.is_shutdown()
        {
            return;
        }
        let queue = self.lock_queue();
        if !queue.is_empty()
            || self.sleeping_tasks.load(Ordering::Acquire) != 0
            || !self
                .active_workers
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .is_empty()
        {
            return;
        }
        self.accepting_timers.store(false, Ordering::Release);
        self.shutdown.store(true, Ordering::Release);
        drop(queue);
        self.close_all_cases();
        self.complete_pending_results();
        self.timers.notify_shutdown();
        let _queue = self.lock_queue();
        self.available.notify_all();
        self.notify_result_change();
    }
}
