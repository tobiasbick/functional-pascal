//! Diagnostic capture owns the domain and releases storage before servicing.
use super::*;
use crate::{
    DomainMaintenance, MaintenanceOutcome, MaintenancePolicy, MemoryDomain, OwnershipError, Value,
};
use std::sync::{Weak, mpsc};
use std::time::Duration;

fn fixture() -> (Arc<CaptureState>, Arc<TaskRecord>, MemoryDomain) {
    let (program, errors) =
        fpas_parser::parse("program Saved; begin const N := 1; discard N; end.");
    assert!(errors.is_empty(), "{errors:?}");
    let mut worker = Worker::new(Arc::new(fpas_compiler::compile(&program).unwrap())).unwrap();
    let state = CaptureState::new(|_| {});
    state.start_root(&mut worker);
    let domain = worker.domain.clone();
    let cell = domain.new_cell(Value::Integer(7)).unwrap();
    worker.store_register(0, Value::Cell(cell)).unwrap();
    let scheduled = state.save(worker.take_task_state(), false);
    let ScheduledTask::Observed(record) = scheduled else {
        panic!("observed state");
    };
    (state, record, domain)
}

#[test]
fn saved_inspection_defers_busy_domains_without_claiming_the_task() {
    struct Paused {
        entered: mpsc::Sender<()>,
        leave: Mutex<mpsc::Receiver<()>>,
    }
    impl DomainMaintenance for Paused {
        fn maintain(&self, _: &MemoryDomain, _: &[Value]) -> Result<(), OwnershipError> {
            self.entered.send(()).unwrap();
            self.leave.lock().unwrap().recv().unwrap();
            Ok(())
        }
    }
    let (state, record, domain) = fixture();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (leave_tx, leave_rx) = mpsc::channel();
    domain.set_maintenance(
        Arc::new(Paused {
            entered: entered_tx,
            leave: Mutex::new(leave_rx),
        }),
        MaintenancePolicy::Explicit,
    );
    std::thread::scope(|scope| {
        let collect = scope.spawn(|| domain.request_explicit_maintenance());
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let snapshot = state.inspect_saved(&record, Instant::now() + Duration::from_secs(1));
        leave_tx.send(()).unwrap();
        assert!(snapshot.is_none());
        assert!(record.saved.lock().unwrap().is_some());
        assert!(matches!(
            collect.join().unwrap(),
            MaintenanceOutcome::Serviced(_)
        ));
    });
    assert!(
        state
            .inspect_saved(&record, Instant::now() + Duration::from_secs(1))
            .is_some()
    );
}

#[test]
fn restored_saved_capture_services_pending_work_only_after_catalog_unlocks() {
    struct Check {
        state: Weak<CaptureState>,
        record: Weak<TaskRecord>,
    }
    impl DomainMaintenance for Check {
        fn maintain(&self, _: &MemoryDomain, _: &[Value]) -> Result<(), OwnershipError> {
            let state = self.state.upgrade().unwrap();
            let record = self.record.upgrade().unwrap();
            assert!(
                state.catalog.template.try_lock().is_ok(),
                "template lock is released"
            );
            assert!(
                record.saved.try_lock().unwrap().is_some(),
                "original task is restored before servicing"
            );
            Ok(())
        }
    }
    let (state, record, domain) = fixture();
    domain.set_maintenance(
        Arc::new(Check {
            state: Arc::downgrade(&state),
            record: Arc::downgrade(&record),
        }),
        MaintenancePolicy::Explicit,
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    std::thread::scope(|scope| {
        // This budget lock is a controlled barrier inside capture, after saved ownership.
        let budget = state.usage.lock().unwrap();
        let inspect = scope.spawn(|| {
            loop {
                if let Some(snapshot) = state.inspect_saved(&record, deadline) {
                    break snapshot;
                }
                // The polling observer may briefly own the same try-locks. Retry a
                // deferred inspection instead of missing the controlled barrier.
                assert!(Instant::now() < deadline, "inspection acquires saved state");
                std::thread::yield_now();
            }
        });
        loop {
            if record.saved.try_lock().is_err()
                && domain.phase() == crate::DomainPhase::ExternalAccess
            {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "inspection reaches budget barrier"
            );
            std::thread::yield_now();
        }
        assert_eq!(
            domain.request_explicit_maintenance(),
            MaintenanceOutcome::Deferred
        );
        drop(budget);
        let _snapshot = inspect.join().unwrap();
    });
    assert!(domain.serviced_generation() > 0);
    assert!(!domain.maintenance_pending());
}

#[test]
fn global_failure_capture_waits_for_exclusive_maintenance() {
    struct Paused {
        entered: mpsc::Sender<()>,
        leave: Mutex<mpsc::Receiver<()>>,
    }
    impl DomainMaintenance for Paused {
        fn maintain(&self, _: &MemoryDomain, _: &[Value]) -> Result<(), OwnershipError> {
            self.entered.send(()).unwrap();
            self.leave.lock().unwrap().recv().unwrap();
            Ok(())
        }
    }
    let (program, errors) = fpas_parser::parse("program Globals; var G: integer := 0; begin end.");
    assert!(errors.is_empty(), "{errors:?}");
    let mut worker =
        Worker::new_with_local_globals(Arc::new(fpas_compiler::compile(&program).unwrap()))
            .unwrap();
    let state = CaptureState::new(|_| {});
    let domain = worker.domain.clone();
    worker.local_globals.as_mut().unwrap()[0] =
        Some(Value::Cell(domain.new_cell(Value::Integer(7)).unwrap()));
    worker.publish_execution(&[]).unwrap();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (leave_tx, leave_rx) = mpsc::channel();
    let (started_tx, started_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    domain.set_maintenance(
        Arc::new(Paused {
            entered: entered_tx,
            leave: Mutex::new(leave_rx),
        }),
        MaintenancePolicy::Explicit,
    );
    std::thread::scope(|scope| {
        let collector = scope.spawn(|| domain.request_explicit_maintenance());
        entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let state_ref = &state;
        let capture = scope.spawn(move || {
            started_tx.send(()).unwrap();
            state_ref.capture_globals(&worker);
            done_tx.send(()).unwrap();
        });
        started_rx.recv_timeout(Duration::from_secs(10)).unwrap();
        assert!(done_rx.try_recv().is_err());
        leave_tx.send(()).unwrap();
        collector.join().unwrap();
        capture.join().unwrap();
    });
    let globals = state.root_globals.lock().unwrap();
    assert_eq!(globals.as_ref().unwrap().bindings.len(), 1);
}
