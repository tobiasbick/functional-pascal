//! Debug-owned task catalog, readiness polling, and single-instruction dispatch.

mod completed_result;
mod completion;
mod recovery;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::Duration;

use fpas_bytecode::Value;

use super::super::types::{DebugTask, DebugTaskEvent, DebugTaskEventKind, DebugTaskState};
use crate::vm::VmError;
use crate::vm::dispatch::DispatchStep;
use crate::vm::tasks::{DebugClock, TaskScheduler, TaskSuspensionState};
use crate::vm::worker::Worker;

pub(in crate::vm::debug) use completed_result::CompletedResultTargetError;

struct TaskSlot {
    worker: Worker,
    entry_function: fpas_bytecode::FunctionId,
    name: String,
    state: DebugTaskState,
    failure: Option<VmError>,
    exited: bool,
}

/// Result of polling the deterministic task scheduler for its next action.
pub(in crate::vm::debug) enum DebugSchedule {
    /// One task can execute exactly one instruction.
    Runnable {
        /// Runtime task identity.
        task_id: u64,
        /// Whether the task has just become runnable at a stable boundary.
        resumed_at_boundary: bool,
    },
    /// No task is runnable before the supplied bounded polling delay.
    Idle(Duration),
}

/// Result of dispatching exactly one task instruction.
pub(in crate::vm::debug) enum DebugDispatch {
    /// The task completed an instruction and remains runnable.
    Instruction(u64),
    /// The task cooperatively suspended or yielded.
    Suspended(u64),
    /// The task returned normally.
    Completed {
        /// Runtime task identity.
        task_id: u64,
        /// Whether this is the main task whose result owns session termination.
        main: bool,
    },
}

/// Single-lane owner of all workers in one debug session.
pub(in crate::vm::debug) struct DebugTaskRuntime {
    scheduler: Arc<TaskScheduler>,
    clock: Arc<DebugClock>,
    tasks: BTreeMap<u64, TaskSlot>,
    last_dispatched: u64,
    resumed_at_boundary: BTreeSet<u64>,
    events: Vec<DebugTaskEvent>,
    root_result: Option<Value>,
}

impl DebugTaskRuntime {
    /// Create the debugger-owned task runtime around the main worker.
    pub(in crate::vm::debug) fn new(
        root: Worker,
        scheduler: Arc<TaskScheduler>,
        clock: Arc<DebugClock>,
    ) -> Self {
        let entry_function = root.function;
        let tasks = BTreeMap::from([(
            0,
            TaskSlot {
                worker: root,
                entry_function,
                name: "FPAS main".to_string(),
                state: DebugTaskState::Runnable,
                failure: None,
                exited: false,
            },
        )]);
        Self {
            scheduler,
            clock,
            tasks,
            last_dispatched: 0,
            resumed_at_boundary: BTreeSet::new(),
            events: Vec::new(),
            root_result: None,
        }
    }

    /// Return one retained task worker, including terminal workers.
    pub(in crate::vm::debug) fn worker(&self, task_id: u64) -> Option<&Worker> {
        self.tasks.get(&task_id).map(|slot| &slot.worker)
    }

    /// Return one retained task worker for stopped-state mutation.
    pub(in crate::vm::debug) fn worker_mut(&mut self, task_id: u64) -> Option<&mut Worker> {
        self.tasks.get_mut(&task_id).map(|slot| &mut slot.worker)
    }

    /// Whether a task currently has a stable instruction-boundary snapshot.
    pub(in crate::vm::debug) fn task_is_inspectable(&self, task_id: u64) -> bool {
        self.tasks
            .get(&task_id)
            .is_some_and(|slot| slot.state.is_inspectable())
    }

    /// Return one task's current lifecycle state.
    pub(in crate::vm::debug) fn task_state(&self, task_id: u64) -> Option<DebugTaskState> {
        self.tasks.get(&task_id).map(|slot| slot.state)
    }

    /// Return stable IDs for every task that can be inspected at this stop.
    pub(in crate::vm::debug) fn inspectable_task_ids(&self) -> Vec<u64> {
        self.tasks
            .iter()
            .filter_map(|(&task_id, slot)| slot.state.is_inspectable().then_some(task_id))
            .collect()
    }

    /// Capture the bounded-protocol task catalog in stable ID order.
    pub(in crate::vm::debug) fn catalog(&mut self) -> Vec<DebugTask> {
        self.drain_spawned();
        self.tasks
            .iter()
            .map(|(&id, slot)| DebugTask {
                id,
                name: slot.name.clone(),
                state: slot.state,
                inspectable: slot.state.is_inspectable(),
            })
            .collect()
    }

    /// Select the next runnable task or the bounded idle delay before polling again.
    pub(in crate::vm::debug) fn schedule(
        &mut self,
        preferred: Option<u64>,
    ) -> Result<DebugSchedule, (u64, VmError)> {
        self.drain_spawned();
        self.refresh_readiness()?;
        if let Some(task_id) = preferred.filter(|task_id| {
            self.tasks
                .get(task_id)
                .is_some_and(|slot| slot.state == DebugTaskState::Runnable)
        }) {
            return Ok(DebugSchedule::Runnable {
                task_id,
                resumed_at_boundary: self.resumed_at_boundary.remove(&task_id),
            });
        }
        let mut runnable = self
            .tasks
            .iter()
            .filter_map(|(&id, slot)| (slot.state == DebugTaskState::Runnable).then_some(id));
        let first = runnable.next();
        let selected = first.and_then(|first| {
            std::iter::once(first)
                .chain(runnable)
                .find(|id| *id > self.last_dispatched)
                .or(Some(first))
        });
        if let Some(task_id) = selected {
            return Ok(DebugSchedule::Runnable {
                task_id,
                resumed_at_boundary: self.resumed_at_boundary.remove(&task_id),
            });
        }
        let wait = self
            .tasks
            .values()
            .filter_map(|slot| match slot.worker.debug_suspension_state() {
                Some(TaskSuspensionState::Sleeping { remaining }) => Some(remaining),
                _ => None,
            })
            .min()
            .unwrap_or(Duration::from_millis(1));
        Ok(DebugSchedule::Idle(wait.min(Duration::from_millis(1))))
    }

    /// Execute exactly one instruction in the selected task.
    pub(in crate::vm::debug) fn dispatch(
        &mut self,
        task_id: u64,
    ) -> Result<DebugDispatch, (u64, VmError)> {
        let Some(slot) = self.tasks.get_mut(&task_id) else {
            unreachable!("scheduled debug task must exist")
        };
        slot.state = DebugTaskState::Running;
        slot.failure = None;
        self.last_dispatched = task_id;
        let dispatch = slot.worker.dispatch_one().map_err(|error| {
            slot.state = DebugTaskState::Failed;
            slot.failure = Some(error.clone());
            if slot.worker.retain_result {
                self.scheduler.store_failure(task_id, error.clone());
            }
            (task_id, error)
        })?;
        let result = match dispatch {
            DispatchStep::Continue => {
                slot.state = DebugTaskState::Runnable;
                DebugDispatch::Instruction(task_id)
            }
            DispatchStep::Suspend => {
                slot.state = state_from_suspension(&slot.worker);
                DebugDispatch::Suspended(task_id)
            }
            DispatchStep::Return(value) => {
                slot.state = DebugTaskState::Completed;
                if task_id == 0 {
                    self.root_result = Some(value.clone());
                }
                if slot.worker.retain_result {
                    self.scheduler.store_result(task_id, value.clone());
                }
                if task_id != 0 {
                    slot.exited = true;
                    self.events.push(DebugTaskEvent {
                        task_id,
                        kind: DebugTaskEventKind::Exited,
                    });
                }
                DebugDispatch::Completed {
                    task_id,
                    main: task_id == 0,
                }
            }
        };
        self.drain_spawned();
        if self.scheduler.is_shutdown() {
            self.cancel_sleeping_tasks();
        }
        Ok(result)
    }

    /// Return the aggregate packed-instruction count for all tasks.
    pub(in crate::vm::debug) fn instruction_count(&self) -> u64 {
        self.tasks.values().fold(0, |total, slot| {
            total.saturating_add(slot.worker.instruction_count)
        })
    }

    #[cfg(test)]
    pub(in crate::vm::debug) fn test_poll_task_result(
        &self,
        task_id: u64,
    ) -> crate::vm::TaskResultPoll {
        self.scheduler.poll_result(task_id)
    }

    /// Drain stable task lifecycle events in creation/completion order.
    pub(in crate::vm::debug) fn take_events(&mut self) -> Vec<DebugTaskEvent> {
        std::mem::take(&mut self.events)
    }

    /// Begin normal root teardown and cancel timer-suspended tasks.
    pub(in crate::vm::debug) fn finish_main(&mut self) {
        self.scheduler.finish_main();
        self.cancel_sleeping_tasks();
    }

    /// Whether every spawned task has reached a terminal lifecycle state.
    pub(in crate::vm::debug) fn spawned_tasks_finished(&self) -> bool {
        self.tasks.iter().all(|(&task_id, slot)| {
            task_id == 0
                || matches!(
                    slot.state,
                    DebugTaskState::Completed | DebugTaskState::Failed | DebugTaskState::Cancelled
                )
        })
    }

    /// Take the root result once normal teardown has finished every spawned task.
    pub(in crate::vm::debug) fn take_finished_root_result(&mut self) -> Option<Value> {
        self.spawned_tasks_finished()
            .then(|| self.root_result.take())
            .flatten()
    }

    /// Wait for the next scheduler poll using the configured monotonic clock.
    pub(in crate::vm::debug) fn wait(&self, duration: Duration) {
        self.clock.wait(duration);
    }

    /// Cancel every remaining task immediately for explicit disconnect or fatal teardown.
    pub(in crate::vm::debug) fn cancel(&mut self) {
        self.scheduler.request_cancel();
        for (&task_id, slot) in &mut self.tasks {
            if !matches!(
                slot.state,
                DebugTaskState::Completed | DebugTaskState::Failed | DebugTaskState::Cancelled
            ) {
                slot.state = DebugTaskState::Cancelled;
            }
            if task_id != 0 && !slot.exited {
                slot.exited = true;
                self.events.push(DebugTaskEvent {
                    task_id,
                    kind: DebugTaskEventKind::Exited,
                });
            }
        }
    }

    fn refresh_readiness(&mut self) -> Result<(), (u64, VmError)> {
        for (&task_id, slot) in &mut self.tasks {
            if !matches!(
                slot.state,
                DebugTaskState::Waiting | DebugTaskState::Sleeping | DebugTaskState::Runnable
            ) || slot.worker.task_suspension.is_none()
            {
                continue;
            }
            match slot.worker.poll_debug_suspension() {
                Ok(true) => {
                    slot.state = DebugTaskState::Runnable;
                    self.resumed_at_boundary.insert(task_id);
                }
                Ok(false) => slot.state = state_from_suspension(&slot.worker),
                Err(error) => {
                    slot.state = DebugTaskState::Failed;
                    slot.failure = Some(error.clone());
                    if slot.worker.retain_result {
                        self.scheduler.store_failure(task_id, error.clone());
                    }
                    return Err((task_id, error));
                }
            }
        }
        Ok(())
    }

    fn drain_spawned(&mut self) {
        while let Some(task) = self.scheduler.try_dequeue() {
            let task_id = task.id;
            let function = task.function;
            let Some(template) = self.tasks.get(&0).map(|slot| &slot.worker) else {
                break;
            };
            let worker = template.worker_for_task(task);
            let function_name = worker
                .executable
                .executable()
                .functions
                .get(usize::from(function.get()))
                .and_then(|info| worker.executable.executable().strings.get(info.name))
                .unwrap_or("<task>")
                .to_string();
            self.tasks.insert(
                task_id,
                TaskSlot {
                    worker,
                    entry_function: function,
                    name: format!("FPAS task {task_id}: {function_name}"),
                    state: DebugTaskState::Runnable,
                    failure: None,
                    exited: false,
                },
            );
            self.resumed_at_boundary.insert(task_id);
            self.events.push(DebugTaskEvent {
                task_id,
                kind: DebugTaskEventKind::Started,
            });
        }
    }

    fn cancel_sleeping_tasks(&mut self) {
        for (&task_id, slot) in &mut self.tasks {
            if slot.state != DebugTaskState::Sleeping {
                continue;
            }
            slot.state = DebugTaskState::Cancelled;
            slot.exited = true;
            slot.worker.task_suspension = None;
            if slot.worker.retain_result {
                self.scheduler.cancel_result(task_id);
            }
            self.events.push(DebugTaskEvent {
                task_id,
                kind: DebugTaskEventKind::Exited,
            });
        }
    }
}

fn state_from_suspension(worker: &Worker) -> DebugTaskState {
    match worker.debug_suspension_state() {
        Some(TaskSuspensionState::Waiting) => DebugTaskState::Waiting,
        Some(TaskSuspensionState::Sleeping { .. }) => DebugTaskState::Sleeping,
        Some(TaskSuspensionState::Yielded) | None => DebugTaskState::Runnable,
    }
}
