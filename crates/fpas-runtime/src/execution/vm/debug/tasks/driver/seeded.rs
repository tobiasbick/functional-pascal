//! Opt-in fair seeded rounds and quiescence-driven virtual task time.

use super::*;
use crate::execution::vm::debug::scheduling::{DebugScheduleReport, rounds::SeededRounds};

impl DebugTaskRuntime {
    pub(in crate::execution::vm::debug) fn enable_seeded(&mut self, seed: u64) {
        let rounds = SeededRounds::new(seed);
        for slot in self.tasks.values_mut() {
            slot.worker.debug_schedule = Some(Arc::clone(&rounds.guard));
        }
        self.seeded = Some(rounds);
    }

    pub(in crate::execution::vm::debug) fn schedule_report(&self) -> DebugScheduleReport {
        let Some(rounds) = &self.seeded else {
            return DebugScheduleReport::requested(None);
        };
        let mut report = DebugScheduleReport::requested(Some(rounds.seed));
        report.status = if rounds.guard.rejection().is_some() {
            "rejected"
        } else if rounds.rounds == 0 {
            "not_started"
        } else {
            "controlled"
        };
        if report.status == "rejected" {
            report.reasons.push("unsupported_effect");
        }
        report.rounds = Some(rounds.rounds.to_string());
        report.dispatches = Some(rounds.dispatches.to_string());
        report.virtual_time_millis = Some(self.clock.now_millis().to_string());
        report
    }

    pub(in crate::execution::vm::debug) fn scheduling_rejection(&self) -> Option<(u64, VmError)> {
        self.seeded.as_ref()?.guard.rejection()
    }

    pub(super) fn seeded_schedule(&mut self) -> Result<DebugSchedule, (u64, VmError)> {
        loop {
            self.drain_spawned();
            let progress = self.refresh_readiness()?;
            if let Some(rejection) = self.scheduling_rejection() {
                return Err(rejection);
            }
            let Some(rounds) = self.seeded.as_mut() else {
                unreachable!("seeded policy installed");
            };
            if let Some(task_id) = rounds.next() {
                if self
                    .tasks
                    .get(&task_id)
                    .is_some_and(TaskSlot::is_schedulable)
                {
                    return Ok(DebugSchedule::Runnable {
                        task_id,
                        resumed_at_boundary: self.resumed_at_boundary.remove(&task_id),
                    });
                }
                continue;
            }
            if rounds.round_dispatched {
                rounds.round_dispatched = false;
                if !self.advance_virtual(Duration::from_millis(1)) {
                    return Ok(DebugSchedule::ClockExhausted);
                }
                continue;
            }
            let ids: Vec<_> = self
                .tasks
                .iter()
                .filter_map(|(&id, slot)| slot.is_schedulable().then_some(id))
                .collect();
            if !ids.is_empty() {
                self.seeded
                    .as_mut()
                    .unwrap_or_else(|| unreachable!("seeded policy installed"))
                    .begin(ids);
                continue;
            }
            // A readiness pass may publish results, cancel scopes, or finish cleanup.
            // Settle those transitions before concluding no internal work remains.
            if progress {
                continue;
            }
            if let Some(wait) = self
                .tasks
                .values()
                .filter_map(TaskSlot::unpaused_sleep_remaining)
                .min()
            {
                if wait.is_zero() || !self.advance_virtual(wait) {
                    return Ok(DebugSchedule::ClockExhausted);
                }
                continue;
            }
            if self
                .tasks
                .values()
                .any(|slot| slot.paused && slot.state.is_inspectable())
            {
                return Ok(DebugSchedule::NoUnpausedWork);
            }
            return Ok(DebugSchedule::Deadlock);
        }
    }

    fn advance_virtual(&self, wait: Duration) -> bool {
        let before = self.clock.now_millis();
        self.clock.wait(wait);
        self.clock.now_millis() > before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn held_ids_are_skipped_and_mid_round_spawns_wait_for_the_next_round() {
        let (program, errors) = fpas_parser::parse("program RoundFixture; begin end.");
        assert!(errors.is_empty());
        let image = Arc::new(fpas_compiler::compile(&program).unwrap());
        let clock = Arc::new(TaskClock::manual());
        let scheduler = Arc::new(TaskScheduler::new());
        let root = Worker::new(Arc::clone(&image))
            .unwrap()
            .with_scheduler(Some(Arc::clone(&scheduler)))
            .with_debug_tasks(Arc::clone(&clock));
        let entry = root.function;
        let mut runtime = DebugTaskRuntime::new(root, scheduler, clock);
        runtime.enable_seeded(42);
        runtime.test_enqueue_pending_task(entry);
        runtime.test_enqueue_pending_task(entry);
        let DebugSchedule::Runnable { task_id: first, .. } = runtime.schedule(None).unwrap() else {
            panic!("first turn");
        };
        runtime.dispatch(first).unwrap();
        let paused = (0..3).find(|id| *id != first).unwrap();
        runtime.pause_task(paused).unwrap();
        runtime.test_enqueue_pending_task(entry);
        let DebugSchedule::Runnable {
            task_id: second, ..
        } = runtime.schedule(None).unwrap()
        else {
            panic!("remaining original turn");
        };
        assert!(second < 3 && second != paused && second != first);
        assert_eq!(runtime.schedule_report().rounds.as_deref(), Some("1"));
        runtime.dispatch(second).unwrap();
        let DebugSchedule::Runnable { task_id: third, .. } = runtime.schedule(None).unwrap() else {
            panic!("new round");
        };
        assert_eq!(third, 3);
        let report = runtime.schedule_report();
        assert_eq!(report.rounds.as_deref(), Some("2"));
        assert_eq!(report.dispatches.as_deref(), Some("2"));
        assert_eq!(report.virtual_time_millis.as_deref(), Some("1"));
    }
}
